use indexmap::IndexMap;

use crate::parser::byte_string::ByteString;
use std::error::Error;
use std::{fmt::Display, fs, iter::Peekable};

#[derive(Debug, PartialEq, Eq, Clone)]
pub enum Bencode {
    // Bencode text is always represented as byte strings
    Text(ByteString),
    Number(u64),
    List(Vec<Self>),
    Dict(IndexMap<ByteString, Self>),
}

#[derive(Debug, Clone)]
pub struct BencodeError {
    message: String,
}

impl BencodeError {
    pub fn new(message: String) -> Self {
        Self { message }
    }
}

impl Error for BencodeError {}

impl Display for BencodeError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.message)
    }
}

pub struct BencodeParser;

impl BencodeParser {
    /// Parse the given raw content to a Bencode value
    pub fn decode(raw_content: &[u8]) -> Result<Bencode, BencodeError> {
        let mut iterator = raw_content.iter().peekable();
        Self::parse(&mut iterator)
    }

    pub fn from_file(path: &str) -> Result<Bencode, BencodeError> {
        let bytes = fs::read(path);
        match bytes {
            Ok(bytes) => Self::decode(&bytes),
            _ => Err(BencodeError::new("invalid file contents".to_string())),
        }
    }

    pub fn encode(value: &Bencode) -> Vec<u8> {
        match value {
            Bencode::Dict(d) => Self::encode_dict(d),
            Bencode::List(l) => Self::encode_list(l),
            Bencode::Number(n) => Self::encode_number(n),
            Bencode::Text(t) => Self::encode_text(t),
        }
    }

    fn encode_number(value: &u64) -> Vec<u8> {
        format!("i{}e", value).as_bytes().to_vec()
    }

    fn encode_text(value: &ByteString) -> Vec<u8> {
        let len = value.len().to_string();
        let mut vec = Vec::new();
        vec.extend(len.as_bytes());
        vec.extend(":".as_bytes());
        vec.extend(value.0.clone());
        vec
    }

    fn encode_list(values: &Vec<Bencode>) -> Vec<u8> {
        let mut vec = Vec::new();
        vec.extend("l".as_bytes());
        for value in values {
            let encoded_value = Self::encode(value);
            vec.extend(encoded_value);
        }
        vec.extend("e".as_bytes());
        vec
    }

    fn encode_dict(value: &IndexMap<ByteString, Bencode>) -> Vec<u8> {
        let mut vec = Vec::new();
        vec.extend("d".as_bytes());
        for (key, value) in value.into_iter() {
            let encoded_value = Self::encode(value);
            let encoded_key = Self::encode_text(key);
            vec.extend(encoded_key);
            vec.extend(encoded_value);
        }

        vec.extend("e".as_bytes());
        vec
    }

    fn parse<'a>(
        iterator: &mut Peekable<impl Iterator<Item = &'a u8>>,
    ) -> Result<Bencode, BencodeError> {
        if let Some(&byte) = iterator.next() {
            return match char::from_u32(byte as u32) {
                Some('i') => Self::parse_int(iterator),
                Some('l') => Self::parse_list(iterator),
                Some('d') => Self::parse_dict(iterator),
                Some(c) if Self::is_digit(c) => Self::parse_str(c, iterator),
                Some(c) => Err(BencodeError::new(format!(
                    "Invalid byte for bencode value: '{}'",
                    c
                ))),
                None => Err(BencodeError::new(
                    "Empty bytes while trying to parse bencode value".to_string(),
                )),
            };
        }

        Err(BencodeError::new(String::from("Invalid Bencode content")))
    }

    fn parse_dict<'a>(
        iterator: &mut Peekable<impl Iterator<Item = &'a u8>>,
    ) -> Result<Bencode, BencodeError> {
        let mut map = IndexMap::new();

        while let Some(&b) = iterator.next() {
            match char::from_u32(b as u32) {
                Some(c) if Self::is_digit(c) => {
                    // we first handle the dictionary key
                    if let Bencode::Text(text) = Self::parse_str(c, iterator)? {
                        // Value can be anything, including dictionaries
                        let value = Self::parse(iterator)?;
                        map.insert(text, value);
                    } else {
                        return Err(BencodeError::new(format!("Invalid string byte {}", c)));
                    }
                }
                // Closing the dictionary
                Some('e') => break,
                Some(c) => {
                    return Err(BencodeError::new(format!(
                        "Invalid string byte for dict length '{}'",
                        c
                    )))
                }
                None => return Err(BencodeError::new("Empty byte for dict key".to_string())),
            }
        }

        Ok(Bencode::Dict(map))
    }

    fn parse_list<'a>(
        iterator: &mut Peekable<impl Iterator<Item = &'a u8>>,
    ) -> Result<Bencode, BencodeError> {
        let mut acc = Vec::new();
        while let Some(&byte) = iterator.next() {
            match char::from_u32(byte as u32) {
                // nested list
                Some('l') => {
                    let list = Self::parse_list(iterator)?;
                    acc.push(list);
                }
                // dictionary
                Some('d') => {
                    let dict = Self::parse_dict(iterator)?;
                    acc.push(dict);
                }
                // integers
                Some('i') => {
                    let number = Self::parse_int(iterator)?;
                    acc.push(number);
                }
                // strings
                Some(c) if Self::is_digit(c) => {
                    let str = Self::parse_str(c, iterator)?;
                    acc.push(str);
                }
                // end of list, closing it
                Some('e') => break,
                Some(c) => return Err(BencodeError::new(format!("Invalid char {}", c))),
                None => break,
            }
        }

        Ok(Bencode::List(acc))
    }

    /// Whether the given character is a valid number character
    fn is_digit(c: char) -> bool {
        ('0'..='9').contains(&c)
    }

    fn parse_str<'a>(
        length_start: char,
        mut iterator: &mut impl Iterator<Item = &'a u8>,
    ) -> Result<Bencode, BencodeError> {
        let mut str_len = Vec::new();
        str_len.push(length_start);

        // First we need to read the string length until we reach the `:`.
        for byte in &mut iterator {
            match char::from_u32(*byte as u32) {
                Some(c) if Self::is_digit(c) => str_len.push(c),
                Some(c) if c == ':' => break,
                Some(c) => {
                    return Err(BencodeError::new(format!(
                        "invalid string length character: '{}'",
                        c
                    )))
                }
                None => return Err(BencodeError::new(String::from("Invalid string value"))),
            }
        }

        // Now we know the string length, we can consume the iterator
        // precisely to the point where the string ends.
        match str_len.iter().collect::<String>().parse::<u64>() {
            Ok(str_len) => {
                let mut str_value = Vec::with_capacity(str_len as usize);

                for byte in iterator.take(str_len as usize) {
                    str_value.push(*byte);
                }

                Ok(Bencode::Text(ByteString::from_vec(str_value)))
            }
            Err(_) => Err(BencodeError::new(format!(
                "Invalid string length '{:?}'",
                str_len
            ))),
        }
    }

    fn parse_int<'a>(
        iterator: &mut Peekable<impl Iterator<Item = &'a u8>>,
    ) -> Result<Bencode, BencodeError> {
        let mut acc = Vec::new();
        while let Some(&byte) = iterator.next() {
            match char::from_u32(byte as u32) {
                Some(c) if Self::is_digit(c) => acc.push(c),
                Some('e') => break,
                Some(c) => {
                    return Err(BencodeError::new(format!(
                        "invalid char '{}' when parsing integers",
                        c
                    )))
                }
                None => break,
            }
        }
        let text_num: String = acc.iter().collect();
        text_num
            .parse::<u64>()
            .map(Bencode::Number)
            .or(Err(BencodeError::new(format!(
                "invalid integer value '{}'",
                text_num
            ))))
    }
}

#[cfg(test)]
mod tests {

    use super::*;

    #[test]
    fn should_parse_integer_values() {
        let str = "64520998877";
        let content = format!("i{}e", str).as_bytes().to_vec();
        let result = BencodeParser::decode(&content).unwrap();
        assert_eq!(result, Bencode::Number(str.parse::<u64>().unwrap()));
    }

    #[test]
    fn should_parse_string_values() {
        let bencode_str = "6:bruno0".as_bytes().to_vec();
        let result = BencodeParser::decode(&bencode_str).unwrap();

        assert_eq!(result, Bencode::Text(ByteString::new("bruno0")));
    }

    #[test]
    fn should_parse_list_of_strings() {
        let list = "l4:spam4:eggse".as_bytes().to_vec();
        let result = BencodeParser::decode(&list).unwrap();
        let expected = Bencode::List(vec![
            Bencode::Text(ByteString::new("spam")),
            Bencode::Text(ByteString::new("eggs")),
        ]);

        assert_eq!(result, expected);
    }

    #[test]
    fn should_parse_list_of_strings_and_integers() {
        let list = "l4:spami55ee".as_bytes().to_vec();
        let result = BencodeParser::decode(&list).unwrap();
        let expected = Bencode::List(vec![
            Bencode::Text(ByteString::new("spam")),
            Bencode::Number(55),
        ]);

        assert_eq!(result, expected);
    }

    #[test]
    fn should_parse_lists_recursively() {
        let list = "l4:spami55eli10el4:spam4:feeti33ee5:brunoee"
            .as_bytes()
            .to_vec();
        let result = BencodeParser::decode(&list).unwrap();

        let expected = Bencode::List(vec![
            Bencode::Text(ByteString::new("spam")),
            Bencode::Number(55),
            Bencode::List(vec![
                Bencode::Number(10),
                Bencode::List(vec![
                    Bencode::Text(ByteString::new("spam")),
                    Bencode::Text(ByteString::new("feet")),
                    Bencode::Number(33),
                ]),
                Bencode::Text(ByteString::new("bruno")),
            ]),
        ]);

        assert_eq!(result, expected);
    }

    #[test]
    fn should_parse_all_value_types_within_a_list() {
        // Readable list
        // let list = r#"
        //   l
        //     i32e
        //     5:bruno
        //     d
        //       4:life
        //       7:is-good
        //       3:age
        //       i64e
        //       4:list
        //       l
        //         i32e
        //         4:cool
        //       e
        //     e
        //   e
        // "#;
        let list = "li32e5:brunod4:life7:is-good3:agei64e4:listli32e4:cooleee"
            .as_bytes()
            .to_vec();
        let result = BencodeParser::decode(&list).unwrap();

        let expected = Bencode::List(vec![
            Bencode::Number(32),
            Bencode::Text(ByteString::new("bruno")),
            Bencode::Dict(IndexMap::from([
                (
                    ByteString::new("life"),
                    Bencode::Text(ByteString::new("is-good")),
                ),
                (ByteString::new("age"), Bencode::Number(64)),
                (
                    ByteString::new("list"),
                    Bencode::List(vec![
                        Bencode::Number(32),
                        Bencode::Text(ByteString::new("cool")),
                    ]),
                ),
            ])),
        ]);

        assert_eq!(result, expected);
    }

    #[test]
    fn should_parse_dictionary() {
        let list =
            "d9:publisher3:bob17:publisher-webpage15:www.example.com18:publisher.location4:home13:publisher.agei33ee"
                .as_bytes()
                .to_vec();
        let result = BencodeParser::decode(&list).unwrap();

        let expected = Bencode::Dict(IndexMap::from([
            (
                ByteString::new("publisher"),
                Bencode::Text(ByteString::new("bob")),
            ),
            (
                ByteString::new("publisher-webpage"),
                Bencode::Text(ByteString::new("www.example.com")),
            ),
            (
                ByteString::new("publisher.location"),
                Bencode::Text(ByteString::new("home")),
            ),
            (ByteString::new("publisher.age"), Bencode::Number(33)),
        ]));

        assert_eq!(result, expected);
    }

    #[test]
    fn should_parse_dictionaries_recursively() {
        let list = "d3:cow3:moo4:spam4:eggs4:home6:vienna3:agei33e4:lifed6:can.be7:amazingee"
            .as_bytes()
            .to_vec();
        let result = BencodeParser::decode(&list).unwrap();

        let expected = Bencode::Dict(IndexMap::from([
            (
                ByteString::new("cow"),
                Bencode::Text(ByteString::new("moo")),
            ),
            (
                ByteString::new("spam"),
                Bencode::Text(ByteString::new("eggs")),
            ),
            (
                ByteString::new("home"),
                Bencode::Text(ByteString::new("vienna")),
            ),
            (ByteString::new("age"), Bencode::Number(33)),
            (
                ByteString::new("life"),
                Bencode::Dict(IndexMap::from([(
                    ByteString::new("can.be"),
                    Bencode::Text(ByteString::new("amazing")),
                )])),
            ),
        ]));

        assert_eq!(result, expected);
    }

    #[test]
    fn should_encode_and_decode_bencode_values_to_bytes() {
        let decoded_value = Bencode::Dict(IndexMap::from([
            (
                ByteString::new("cow"),
                Bencode::Text(ByteString::new("moo")),
            ),
            (
                ByteString::new("spam"),
                Bencode::Text(ByteString::new("eggs")),
            ),
            (
                ByteString::new("home"),
                Bencode::Text(ByteString::new("vienna")),
            ),
            (ByteString::new("age"), Bencode::Number(33)),
            (
                ByteString::new("life"),
                Bencode::Dict(IndexMap::from([(
                    ByteString::new("can.be"),
                    Bencode::Text(ByteString::new("amazing")),
                )])),
            ),
            (
                ByteString::new("items"),
                Bencode::List(vec![
                    Bencode::Number(10),
                    Bencode::Text(ByteString::new("who cares")),
                ]),
            ),
        ]));

        let encoded_value = BencodeParser::encode(&decoded_value);

        let new_decoded_value = BencodeParser::decode(&encoded_value).unwrap();
        assert_eq!(decoded_value, new_decoded_value);
    }
}
