use rustorrent::parser::{
    bencode::BencodeParser,
    meta_info::MetaInfo,
    meta_info::{FileMode, SingleFile},
};

#[test]
fn can_parse_bencode_from_file() {
    let content = BencodeParser::from_file("tests/ubuntu_sample.torrent");
    assert!(content.is_ok());
}

#[test]
fn can_decode_a_torrent_file_with_a_single_file() {
    let meta_info = MetaInfo::from_file("tests/ubuntu_sample.torrent");
    assert!(&meta_info.is_ok());

    let meta_info = meta_info.unwrap();
    assert_eq!(&meta_info.announce, "https://torrent.ubuntu.com/announce");
    assert_eq!(
        &meta_info.info.file_info,
        &FileMode::Single(SingleFile {
            length: 4071903232,
            md5sum: None,
            name: String::from("ubuntu-22.10-desktop-amd64.iso"),
        })
    );
}

// @TODO: write a test that accepts .torrent files with multi-file mode
