use rustorrent::parser::bencode::BencodeParser;
use std::fs;

#[test]
fn can_decode_torrent_file_contents() {
    let torrent_bytes = fs::read("tests/ubuntu_sample.torrent").unwrap();
    let parsed_content = BencodeParser::decode(&torrent_bytes);
    assert!(parsed_content.is_ok());
}
