use super::bencode::BencodeError;
use super::byte_string::ByteString;

/// Meta-info files (.torrent) according to the (unofficial) spec.
/// See: https://wiki.theory.org/BitTorrentSpecification#Metainfo_File_Structure
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

pub enum FileMode {
    SingleFile(File),
    MultiFile(FileList),
}

pub struct FileList {
    /// the name of the directory in which to store all the files.
    /// This is purely advisory. (string)
    pub name: String,
    pub files: Vec<File>,
}

pub struct File {
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
    pub path: String,
}

impl MetaInfo {
    /// decode .torrent files as a readable metainfo data structure
    pub fn from_file(_path: &str) -> Result<Self, BencodeError> {
        panic!("missing impl")
    }
}
