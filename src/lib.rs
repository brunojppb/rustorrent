use std::{collections::HashMap, fmt::Display, iter::Peekable};

#[derive(Debug)]
pub struct ByteString(Vec<u8>);

impl Display for ByteString {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        if let Ok(text) = String::from_utf8(self.0.clone()) {
            // For strings that are UTF-8 encoded, we can safely format them
            write!(f, "{}", text)
        } else {
            // For raw strings, we can just display the raw bytes
            write!(f, "{:?}", self.0)
        }
    }
}

#[derive(Debug)]
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
        while let Some(byte) = iterator.next() {
            match char::from_u32(*byte as u32) {
                Some('i') => return Self::parse_int(iterator),
                Some(c) if Self::is_digit(c) => {
                    panic!("String handling not implemented")
                }
                _ => panic!("Match arm not implemented yet"),
                // Some('l') => println!("Starting a List"),
                // Some('d') => println!("Starting a Dict"),
                // Some(c) => println!("Continue value {}", c),
                // None => println!("Got nothing from {}", byte),
            }
        }

        Err(ParsingError::new(String::from("Invalid Bencode content")))
    }

    /// Whether the given character is a valid number character
    fn is_digit(c: char) -> bool {
        c >= '0' && c <= '9'
    }

    fn parse_int<'a>(
        iterator: &mut Peekable<impl Iterator<Item = &'a u8>>,
    ) -> Result<Bencode, ParsingError> {
        let mut acc = Vec::new();
        while let Some(byte) = iterator.next() {
            match char::from_u32(*byte as u32) {
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
        let content = "i64520998877e".as_bytes().to_vec();
        let result = BencodeParser::decode(&content).unwrap();
        assert!(matches!(result, Bencode::Number(64520998877)));
    }

    // #[test]
    // fn parse_torrent_file() {
    //     let content = fs::read("ubuntu_iso.torrent").unwrap();
    //     let meta_info = Bencode::parse(&content);
    //     assert_eq!(1, 1);
    //     // assert_eq!(meta_info.announce, "https://torrent.ubuntu.com/announce");
    // }
}
