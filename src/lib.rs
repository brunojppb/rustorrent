use std::{collections::HashMap, fmt::Display};

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

pub enum BencodeValue {
    // Bencode text is always represented as byte strings
    Text(ByteString),
    Number(i64),
    List(Vec<Self>),
    Dict(HashMap<String, Self>),
}

pub struct Bencode;

impl Bencode {
    /// Parse the given raw content to a Bencode value
    pub fn parse(raw_content: &Vec<u8>) {
        let mut iterator = raw_content.iter().peekable();
        while let Some(byte) = iterator.next() {
            match char::from_u32(*byte as u32) {
                Some('i') => println!("Start integer"),
                Some('l') => println!("Start list"),
                Some('d') => println!("Start Dict"),
                Some(c) => println!("Continue value {}", c),
                None => println!("Got nothing from {}", byte),
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use std::fs;

    use super::*;

    #[test]
    fn should_parse_integer_values() {
        let content = "i645e".as_bytes().to_vec();
        let result = Bencode::parse(&content);
    }

    // #[test]
    // fn parse_torrent_file() {
    //     let content = fs::read("ubuntu_iso.torrent").unwrap();
    //     let meta_info = Bencode::parse(&content);
    //     assert_eq!(1, 1);
    //     // assert_eq!(meta_info.announce, "https://torrent.ubuntu.com/announce");
    // }
}
