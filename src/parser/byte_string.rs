use std::{
    fmt::{Debug, Display},
    hash::Hash,
    ops::Deref,
};

#[derive(Hash, Clone, Eq)]
pub struct ByteString(pub Vec<u8>);

/// a ByteString is just a string of bytes. It does not have encoding information.
///
/// Note: We can try to decode it as UTF-8 when calling `to_string` but we fallback
/// to just show the byte array length instead if that fails.
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
        if let Ok(text) = core::str::from_utf8(&self.0) {
            // For strings that are UTF-8 encoded, we can safely format them
            write!(f, "{}", text)
        } else {
            // For raw strings, we can just display the raw array size for now
            write!(f, "{:?}", self.0.len())
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

impl PartialEq for ByteString {
    fn eq(&self, other: &Self) -> bool {
        Self::compare_vectors(self, other)
    }

    fn ne(&self, other: &Self) -> bool {
        !Self::compare_vectors(self, other)
    }
}
