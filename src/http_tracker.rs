use crate::parser::announce_info::AnnounceInfo;
use crate::parser::{bencode::BencodeParser, meta_info::Info};
use sha1::{Digest, Sha1};

/// Handle HTTP trackers providing torrent information.
/// Mostly following the (unofficial) spec from [wiki.theory.org](https://wiki.theory.org/BitTorrentSpecification#Tracker_Request_Parameters)
pub struct HTTPTracker<'a> {
    peer_id: &'a str,
}

impl<'a> HTTPTracker<'a> {
    pub fn new(peer_id: &'a str) -> Self {
        Self { peer_id }
    }

    pub async fn get_announce_info(
        &self,
        url: &str,
        info: Info,
    ) -> Result<AnnounceInfo, Box<dyn std::error::Error>> {
        let info_hash = Self::generate_hash(&info.bencode_value);
        // @TODO: generate a peer ID during client boot?
        // Probably read something from the build config and
        // use some sort of string generator as a suffix
        let peer_id = Self::generate_hash(&self.peer_id.as_bytes().to_vec());

        let client = reqwest::Client::new();
        // when using reqwest query methods, the info_hash and peer_id
        // will be URL encoded again, which modifies the binary string.
        // so to keep these query parameters stable, we simply append
        // them to the original URL and use reqwest to manage the other
        // params.
        let url_with_hash = format!("{}?info_hash={}&peer_id={}", url, info_hash, peer_id);

        let response = client
            .get(url_with_hash)
            .query(&[
                ("port", String::from("6889")),
                ("uploaded", String::from("0")),
                ("downloaded", String::from("0")),
                ("left", info.piece_length.to_string()),
                ("compact", String::from("1")),
                ("event", String::from("started")),
            ])
            .send()
            .await?
            .bytes()
            .await?;

        let bencode_resp = BencodeParser::decode(&response.to_vec())?;
        let announce_info = AnnounceInfo::parse(&bencode_resp)?;

        Ok(announce_info)
    }

    fn generate_hash(value: &Vec<u8>) -> String {
        let mut hasher = Sha1::new();
        hasher.update(value);
        let bytes = hasher.finalize();
        urlencoding::encode_binary(&bytes).into_owned()
    }
}

#[cfg(test)]
mod tests {

    use crate::parser::meta_info::MetaInfo;

    use super::*;

    #[tokio::test]
    async fn should_get_announce_server_info_from_torrent_file() {
        let meta_info = MetaInfo::from_file("tests/ubuntu_sample.torrent").unwrap();
        // example of a valid announce URL:
        // https://torrent.ubuntu.com/announce?info_hash=%99%C8%2B%B75%05%A3%C0%B4S%F9%FA%0E%88%1DnZ2%A0%C1&peer_id=%B7%C0%9B%A8%FC%DC%FB%91%C1N%AE%8D%DBZ%E2b%F2%84%B6%E5&port=8888&uploaded=0&downloaded=0&left=555555&compact=1&event=started
        let http_tracker = HTTPTracker::new("rustorrent-client-dev");
        let resp = http_tracker
            .get_announce_info(&meta_info.announce, meta_info.info)
            .await;

        // println!("Resp: {:#?}", resp);

        assert!(resp.is_ok());
    }
}
