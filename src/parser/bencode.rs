use crate::parser::byte_string::ByteString;
use std::{collections::HashMap, fmt::Display, fs, iter::Peekable};

#[derive(Debug, PartialEq, Eq)]
pub enum Bencode {
    // Bencode text is always represented as byte strings
    Text(ByteString),
    Number(u64),
    List(Vec<Self>),
    Dict(HashMap<String, Self>),
}

#[derive(Debug, Clone)]
pub struct BencodeError {
    message: String,
}

impl BencodeError {
    fn new(message: String) -> Self {
        Self { message }
    }
}

impl Display for BencodeError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.message)
    }
}

pub struct BencodeParser;

impl BencodeParser {
    /// Parse the given raw content to a Bencode value
    pub fn decode(raw_content: &Vec<u8>) -> Result<Bencode, BencodeError> {
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

    fn parse<'a>(
        iterator: &mut Peekable<impl Iterator<Item = &'a u8>>,
    ) -> Result<Bencode, BencodeError> {
        while let Some(&byte) = iterator.next() {
            match char::from_u32(byte as u32) {
                Some('i') => return Self::parse_int(iterator),
                Some('l') => return Self::parse_list(iterator),
                Some('d') => return Self::parse_dict(iterator),
                Some(c) if Self::is_digit(c) => return Self::parse_str(c, iterator),
                Some(c) => {
                    return Err(BencodeError::new(format!(
                        "Invalid byte for bencode value: '{}'",
                        c
                    )))
                }
                None => {
                    return Err(BencodeError::new(
                        "Empty bytes while trying to parse bencode value".to_string(),
                    ))
                }
            }
        }

        Err(BencodeError::new(String::from("Invalid Bencode content")))
    }

    fn parse_dict<'a>(
        iterator: &mut Peekable<impl Iterator<Item = &'a u8>>,
    ) -> Result<Bencode, BencodeError> {
        let mut map = HashMap::new();

        while let Some(&byte) = iterator.next() {
            match char::from_u32(byte as u32) {
                Some(c) if Self::is_digit(c) => {
                    // we first handle the dictionary key
                    if let Bencode::Text(text) = Self::parse_str(c, iterator)? {
                        // Value can be anything, including dictionaries
                        let value = Self::parse(iterator)?;
                        map.insert(text.to_string(), value);
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
        c >= '0' && c <= '9'
    }

    fn parse_str<'a>(
        length_start: char,
        iterator: &mut Peekable<impl Iterator<Item = &'a u8>>,
    ) -> Result<Bencode, BencodeError> {
        let mut str_len = Vec::new();
        str_len.push(length_start);
        // First we need to read the string length until we reach the `:`.
        while let Some(&byte) = iterator.next() {
            match char::from_u32(byte as u32) {
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
                let mut str_value = Vec::new();

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
            .map(|int| Bencode::Number(int))
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
            Bencode::Dict(HashMap::from([
                (
                    "life".to_string(),
                    Bencode::Text(ByteString::new("is-good")),
                ),
                ("age".to_string(), Bencode::Number(64)),
                (
                    "list".to_string(),
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

        let expected = Bencode::Dict(HashMap::from([
            (
                "publisher".to_string(),
                Bencode::Text(ByteString::new("bob")),
            ),
            (
                "publisher-webpage".to_string(),
                Bencode::Text(ByteString::new("www.example.com")),
            ),
            (
                "publisher.location".to_string(),
                Bencode::Text(ByteString::new("home")),
            ),
            ("publisher.age".to_string(), Bencode::Number(33)),
        ]));

        assert_eq!(result, expected);
    }

    #[test]
    fn should_parse_dictionaries_recursively() {
        let list = "d3:cow3:moo4:spam4:eggs4:home6:vienna3:agei33e4:lifed6:can.be7:amazingee"
            .as_bytes()
            .to_vec();
        let result = BencodeParser::decode(&list).unwrap();

        let expected = Bencode::Dict(HashMap::from([
            ("cow".to_string(), Bencode::Text(ByteString::new("moo"))),
            ("spam".to_string(), Bencode::Text(ByteString::new("eggs"))),
            ("home".to_string(), Bencode::Text(ByteString::new("vienna"))),
            ("age".to_string(), Bencode::Number(33)),
            (
                "life".to_string(),
                Bencode::Dict(HashMap::from([(
                    "can.be".to_string(),
                    Bencode::Text(ByteString::new("amazing")),
                )])),
            ),
        ]));

        assert_eq!(result, expected);
    }
}
