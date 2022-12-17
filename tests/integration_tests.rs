use rustorrent::parser::bencode::BencodeParser;

#[test]
fn can_decode_torrent_file_contents() {
    let content = BencodeParser::from_file("tests/ubuntu_sample.torrent");
    assert!(content.is_ok());
}
