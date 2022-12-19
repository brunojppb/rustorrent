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

#[test]
fn can_decode_a_torrent_file_with_multiple_files() {
    let meta_info = MetaInfo::from_file("tests/haphead_bundle.torrent");

    println!("Meta Info: {:?}", meta_info);
    assert!(&meta_info.is_ok());

    let meta_info = meta_info.unwrap();
    assert_eq!(
        &meta_info.announce,
        "dht://3C9650FDF0E03236FD7CDB343FFB1F792342C11F.dht/announce"
    );

    // @TODO: Assert on file mode content for list of files
}
