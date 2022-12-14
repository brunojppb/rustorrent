use std::{
    fmt::{Debug, Display},
    hash::Hash,
    ops::Deref,
};

#[derive(Hash)]
pub struct ByteString(Vec<u8>);

impl ByteString {
    pub fn new(str: &str) -> Self {
        Self(str.as_bytes().to_vec())
    }

    pub fn from_vec(vec: Vec<u8>) -> Self {
        Self(vec)
    }

    fn compare_vectors(a: &Vec<u8>, b: &Vec<u8>) -> bool {
        let matching = a.iter().zip(b.iter()).filter(|&(a, b)| a == b).count();
        matching == a.len() && matching == b.len()
    }

    fn print(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        if let Ok(text) = String::from_utf8(self.0.clone()) {
            // For strings that are UTF-8 encoded, we can safely format them
            write!(f, "{}", text)
        } else {
            // For raw strings, we can just display the raw bytes
            write!(f, "bytes_length:{:?}", self.0.len())
        }
    }
}

impl Debug for ByteString {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        self.print(f)
    }
}

impl Display for ByteString {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        self.print(f)
    }
}

impl Deref for ByteString {
    type Target = Vec<u8>;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl Eq for ByteString {}

impl PartialEq for ByteString {
    fn eq(&self, other: &Self) -> bool {
        Self::compare_vectors(self, other)
    }

    fn ne(&self, other: &Self) -> bool {
        !Self::compare_vectors(self, other)
    }
}
