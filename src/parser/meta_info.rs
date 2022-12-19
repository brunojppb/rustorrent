use std::collections::HashMap;

use super::bencode::{Bencode, BencodeError, BencodeParser};
use super::byte_string::ByteString;

type Dict = HashMap<String, Bencode>;

/// Meta-info files (.torrent) according to the (unofficial) spec.
/// See: https://wiki.theory.org/BitTorrentSpecification#Metainfo_File_Structure
#[derive(Debug)]
pub struct MetaInfo {
    pub info: Info,
    /// The announce URL of the tracker
    pub announce: String,
    // extension to the official specification, offering backwards-compatibility.
    pub announce_list: Option<Vec<String>>,
    pub creation_date: Option<u64>,
    /// ree-form textual comments of the author
    pub comment: Option<String>,
    pub created_by: Option<String>,
    /// the string encoding format used to generate the pieces part
    /// of the info dictionary in the .torrent metafile
    pub encoding: Option<String>,
}

impl MetaInfo {
    /// Parse the given file (.torrent) in a valid MetaInfo data structure
    pub fn from_file(path: &str) -> Result<Self, BencodeError> {
        let bencode = BencodeParser::from_file(path)?;
        match bencode {
            Bencode::Dict(dict) => {
                let info = Info::from(&dict)?;

                if let Bencode::Text(announce) = get_value("announce", &dict)? {
                    let announce_list = dict.get("announce-list").and_then(|l| match l {
                        Bencode::List(list) => {
                            let res = list
                                .iter()
                                .filter_map(|v| match v {
                                    // Announce list is always a list if lists of strings (Vec<Vec<String>>)
                                    // so we need to flatten them out
                                    Bencode::List(list) => {
                                        let mut values = Vec::with_capacity(list.len());
                                        for text in list.iter() {
                                            if let Bencode::Text(announce_url) = text {
                                                values.push(announce_url.to_string());
                                            }
                                        }
                                        Some(values)
                                    }
                                    _ => None,
                                })
                                .flatten()
                                .collect::<Vec<String>>();
                            Some(res)
                        }
                        _ => None,
                    });
                    let comment = get_optional_str("comment", &dict);
                    let created_by = get_optional_str("created by", &dict);
                    let encoding = get_optional_str("encoding", &dict);
                    let creation_date = dict.get("creation date").and_then(|date| match date {
                        Bencode::Number(date_int) => Some(date_int.clone()),
                        _ => None,
                    });

                    return Ok(Self {
                        info,
                        announce: announce.to_string(),
                        announce_list,
                        comment,
                        created_by,
                        encoding,
                        creation_date,
                    });
                }

                Err(parsing_error("Invalid metainfo file"))
            }
            _ => Err(parsing_error("Invalid metainfo torrent file")),
        }
    }
}

#[derive(Debug)]
pub struct Info {
    /// number of bytes in each piece (integer)
    pub piece_length: u64,
    /// concatenation of all 20-byte SHA1 hash values,
    /// one per piece (byte string, i.e. not urlencoded)
    pub pieces: ByteString,
    /// Whether publish its presence to get other peers ONLY via
    /// the trackers explicitly described in the metainfo file.
    /// If this field is set to "true" or is not present, the client may
    /// obtain peer from other means, e.g. PEX peer exchange, dht.
    /// Here, "private" may be read as "no external peer source".
    pub private: bool,
    pub file_info: FileMode,
}

impl Info {
    fn from(dict: &Dict) -> Result<Self, BencodeError> {
        if let Bencode::Dict(info_dict) = get_value("info", dict)? {
            if let Bencode::Number(piece_length) = get_value("piece length", info_dict)? {
                if let Bencode::Text(pieces) = get_value("pieces", info_dict)? {
                    let private = info_dict
                        .get("private")
                        .map(|v| &Bencode::Number(1) == v)
                        .unwrap_or_else(|| false);
                    let file_info = Self::parse_file_info(info_dict)?;
                    return Ok(Self {
                        piece_length: piece_length.clone(),
                        pieces: pieces.clone(),
                        private,
                        file_info,
                    });
                }
            }
        }
        Err(parsing_error("Invalid meta_info"))
    }

    fn parse_file_info(dict: &Dict) -> Result<FileMode, BencodeError> {
        match dict.get("files") {
            // Multiple files mode
            Some(_) => {
                let multi_file = MultiFile::from(dict)?;
                Ok(FileMode::Multi(multi_file))
            }
            // single-file mode
            None => {
                let single_file = SingleFile::from(dict)?;
                Ok(FileMode::Single(single_file))
            }
        }
    }
}

#[derive(Debug, PartialEq, Eq)]
pub enum FileMode {
    Single(SingleFile),
    Multi(MultiFile),
}

#[derive(Debug, PartialEq, Eq)]
pub struct MultiFile {
    /// the name of the directory in which to store all the files.
    /// This is purely advisory. (string)
    pub name: String,
    pub files: Vec<MultiFileItem>,
}

impl MultiFile {
    fn from(dict: &Dict) -> Result<Self, BencodeError> {
        if let Bencode::Text(name) = get_value("name", dict)? {
            if let Bencode::List(files) = get_value("files", &dict)? {
                let mut file_items = Vec::with_capacity(files.len());
                for file in files {
                    match file {
                        Bencode::Dict(file) => {
                            let file = MultiFileItem::from(&file)?;
                            file_items.push(file);
                        }
                        _ => {
                            return Err(BencodeError::new(String::from("invalid file in metainfo")))
                        }
                    }
                }

                return Ok(Self {
                    name: name.to_string(),
                    files: file_items,
                });
            }
        }
        Err(parsing_error("Invalid multi-file"))
    }
}

#[derive(Debug, PartialEq, Eq)]
pub struct MultiFileItem {
    pub length: u64,
    /// (optional) a 32-character hexadecimal string corresponding
    /// to the MD5 sum of the file. This is not used by BitTorrent at all,
    /// but it is included by some programs for greater compatibility.
    pub md5sum: Option<String>,
    /// a list containing one or more string elements that together
    /// represent the path and filename. Each element in the list corresponds
    /// to either a directory name or (in the case of the final element) the filename.
    /// For example, a the file "dir1/dir2/file.ext" would consist of three string
    /// elements: "dir1", "dir2", and "file.ext". This is encoded as a bencoded list
    /// of strings such as 'l4:dir14:dir28:file.exte'
    pub path: Vec<String>,
}

impl MultiFileItem {
    fn from(dict: &Dict) -> Result<Self, BencodeError> {
        if let Some(path) = get_opt_str_list("path", dict) {
            if let Bencode::Number(length) = get_value("length", dict)? {
                let md5sum = get_optional_str("md5sum", dict);
                return Ok(Self {
                    length: length.clone(),
                    path,
                    md5sum,
                });
            }
        }

        Err(parsing_error(&format!("invalid file item: {:?}", dict)))
    }
}

#[derive(Debug, PartialEq, Eq)]
pub struct SingleFile {
    pub name: String,
    pub length: u64,
    /// (optional) a 32-character hexadecimal string corresponding
    /// to the MD5 sum of the file. This is not used by BitTorrent at all,
    /// but it is included by some programs for greater compatibility.
    pub md5sum: Option<String>,
}

impl SingleFile {
    fn from(dict: &Dict) -> Result<Self, BencodeError> {
        if let Bencode::Text(name) = get_value("name", dict)? {
            if let Bencode::Number(length) = get_value("length", dict)? {
                let md5sum = get_optional_str("md5sum", dict);
                return Ok(Self {
                    name: name.to_string(),
                    length: length.clone(),
                    md5sum,
                });
            }
        }

        Err(parsing_error("invalid file"))
    }
}

fn get_opt_str_list(key: &str, dict: &Dict) -> Option<Vec<String>> {
    dict.get(key).and_then(|v| match v {
        Bencode::List(list) => {
            let mut values = Vec::with_capacity(list.len());
            for code in list.iter() {
                if let Bencode::Text(str) = code {
                    values.push(str.to_string());
                }
            }
            Some(values)
        }
        _ => None,
    })
}

fn get_optional_str(key: &str, dict: &Dict) -> Option<String> {
    dict.get(key).and_then(|v| match v {
        Bencode::Text(value) => Some(value.to_string()),
        _ => None,
    })
}

/// Get a Bencode value from the given hashmap.
fn get_value<'a>(key: &str, dict: &'a Dict) -> Result<&'a Bencode, BencodeError> {
    if let Some(value) = dict.get(key) {
        Ok(value)
    } else {
        println!("Error dict: {:?}", dict);
        Err(BencodeError::new(format!(
            "could not find key '{}' in meta info dict",
            key
        )))
    }
}

fn parsing_error(msg: &str) -> BencodeError {
    BencodeError::new(msg.to_string())
}
