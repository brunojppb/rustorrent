use crate::parser::byte_string::ByteString;
use std::{collections::HashMap, fmt::Display, iter::Peekable};

#[derive(Debug, PartialEq, Eq)]
pub enum Bencode {
    // Bencode text is always represented as byte strings
    Text(ByteString),
    Number(i64),
    List(Vec<Self>),
    Dict(HashMap<String, Self>),
}

#[derive(Debug, Clone)]
pub struct ParsingError {
    message: String,
}

impl ParsingError {
    fn new(message: String) -> Self {
        Self { message }
    }
}

impl Display for ParsingError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.message)
    }
}

pub struct BencodeParser;

impl BencodeParser {
    /// Parse the given raw content to a Bencode value
    pub fn decode(raw_content: &Vec<u8>) -> Result<Bencode, ParsingError> {
        let mut iterator = raw_content.iter().peekable();
        Self::parse(&mut iterator)
    }

    fn parse<'a>(
        iterator: &mut Peekable<impl Iterator<Item = &'a u8>>,
    ) -> Result<Bencode, ParsingError> {
        while let Some(&byte) = iterator.next() {
            match char::from_u32(byte as u32) {
                Some('i') => return Self::parse_int(iterator),
                Some('l') => return Self::parse_list(iterator),
                Some(c) if Self::is_digit(c) => return Self::parse_str(c, iterator),
                _ => panic!("Match arm not implemented yet"),
                // Some('l') => println!("Starting a List"),
                // Some('d') => println!("Starting a Dict"),
                // Some(c) => println!("Continue value {}", c),
                // None => println!("Got nothing from {}", byte),
            }
        }

        Err(ParsingError::new(String::from("Invalid Bencode content")))
    }

    fn parse_list<'a>(
        iterator: &mut Peekable<impl Iterator<Item = &'a u8>>,
    ) -> Result<Bencode, ParsingError> {
        let mut acc = Vec::new();
        while let Some(&byte) = iterator.next() {
            match char::from_u32(byte as u32) {
                // nested list
                Some('l') => {
                    let list = Self::parse_list(iterator)?;
                    acc.push(list);
                }
                // dictionary
                Some('d') => panic!("Dictionary not handled yet"),
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
                Some(c) => return Err(ParsingError::new(format!("Invalid char {}", c))),
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
    ) -> Result<Bencode, ParsingError> {
        let mut str_len = Vec::new();
        str_len.push(length_start);
        // First we need to read the string length until we reach the `:`.
        while let Some(&byte) = iterator.next() {
            match char::from_u32(byte as u32) {
                Some(c) if Self::is_digit(c) => str_len.push(c),
                Some(c) if c == ':' => break,
                Some(c) => {
                    return Err(ParsingError::new(format!(
                        "invalid string length character: '{}'",
                        c
                    )))
                }
                None => return Err(ParsingError::new(String::from("Invalid string value"))),
            }
        }

        // Now we know the string length, we can consume the iterator
        // precisely to the point where the string ends.
        let str_len: String = str_len.iter().collect();
        let str_len: i64 = str_len.parse().unwrap();
        let mut str_value = Vec::new();

        for _ in 0..str_len {
            let byte = iterator.next().unwrap();
            str_value.push(*byte);
        }

        Ok(Bencode::Text(ByteString::from_vec(str_value)))
    }

    fn parse_int<'a>(
        iterator: &mut Peekable<impl Iterator<Item = &'a u8>>,
    ) -> Result<Bencode, ParsingError> {
        let mut acc = Vec::new();
        while let Some(&byte) = iterator.next() {
            match char::from_u32(byte as u32) {
                Some(c) if Self::is_digit(c) => acc.push(c),
                Some('e') => break,
                Some(c) => {
                    return Err(ParsingError::new(format!(
                        "invalid char '{}' when parsing numbers",
                        c
                    )))
                }
                None => break,
            }
        }
        let text_num: String = acc.iter().collect();
        let num: i64 = text_num.parse().unwrap();
        return Ok(Bencode::Number(num));
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
        assert!(result == Bencode::Number(str.parse::<i64>().unwrap()));
    }

    #[test]
    fn should_parse_string_values() {
        let bencode_str = "6:bruno0".as_bytes().to_vec();
        let result = BencodeParser::decode(&bencode_str).unwrap();

        assert!(result == Bencode::Text(ByteString::new("bruno0")));
    }

    #[test]
    fn should_parse_list_of_strings() {
        let list = "l4:spam4:eggse".as_bytes().to_vec();
        let result = BencodeParser::decode(&list).unwrap();
        let expected = Bencode::List(vec![
            Bencode::Text(ByteString::new("spam")),
            Bencode::Text(ByteString::new("eggs")),
        ]);

        assert!(result == expected);
    }

    #[test]
    fn should_parse_list_of_strings_and_integers() {
        let list = "l4:spami55ee".as_bytes().to_vec();
        let result = BencodeParser::decode(&list).unwrap();
        let expected = Bencode::List(vec![
            Bencode::Text(ByteString::new("spam")),
            Bencode::Number(55),
        ]);

        assert!(result == expected);
    }

    #[test]
    fn should_parse_lists_recursively() {
        let list = "l4:spami55eli10el4:spam4:feeti33ee5:brunoee"
            .as_bytes()
            .to_vec();
        let result = BencodeParser::decode(&list).unwrap();

        println!("Result: {:?}", result);

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

        println!("Expected: {:?}", expected);

        assert!(result == expected);
    }
}
