use crate::parser::announce_info::AnnounceInfo;
use crate::parser::{bencode::BencodeParser, meta_info::Info};
use reqwest::Client;
use sha1::{Digest, Sha1};

/// Handle HTTP trackers providing torrent information.
/// Mostly following the (unofficial) spec from [wiki.theory.org](https://wiki.theory.org/BitTorrentSpecification#Tracker_Request_Parameters)
pub struct HTTPTracker<'a> {
    peer_id: &'a str,
    http_client: Client,
}

impl<'a> HTTPTracker<'a> {
    pub fn new(peer_id: &'a str, http_client: Client) -> Self {
        Self {
            peer_id,
            http_client,
        }
    }

    pub async fn get_announce_info(
        &self,
        url: &str,
        info: Info,
    ) -> Result<AnnounceInfo, Box<dyn std::error::Error>> {
        let info_hash = Self::generate_hash(&info.bencode_value);
        // TODO: generate a peer ID during client boot?
        // Probably read something from the build config and
        // use some sort of string generator as a suffix for each user.
        // We should persist this so we can reuse the same client ID
        // between app boots.
        let peer_id = Self::generate_hash(&self.peer_id.as_bytes().to_vec());

        // when using reqwest query methods, the info_hash and peer_id
        // will be URL encoded again, which modifies the binary string.
        // so to keep these query parameters stable, we simply append
        // them to the original URL and use reqwest to manage the other
        // params.
        let url_with_hash = format!("{}?info_hash={}&peer_id={}", url, info_hash, peer_id);

        let response = self
            .http_client
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

        let bencode_resp = BencodeParser::decode(&response)?;
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

    use std::fs;

    use wiremock::ResponseTemplate;

    use crate::parser::meta_info::MetaInfo;

    use super::*;

    #[tokio::test]
    // #[ignore = "Can't test that in CI yet until I have a HTTP client mocking strategy"]
    async fn should_get_announce_server_info_from_torrent_file() {
        let meta_info = MetaInfo::from_file("tests/ubuntu_sample.torrent").unwrap();
        let decoded_announce_response = fs::read("tests/announce_response").unwrap();

        let mock_server = wiremock::MockServer::start().await;

        // Register mock into the mock server
        wiremock::Mock::given(wiremock::matchers::any())
            .respond_with(ResponseTemplate::new(200).set_body_bytes(decoded_announce_response))
            .expect(1)
            .mount(&mock_server)
            .await;

        // example of a valid announce URL:
        // https://torrent.ubuntu.com/announce?info_hash=%99%C8%2B%B75%05%A3%C0%B4S%F9%FA%0E%88%1DnZ2%A0%C1&peer_id=%B7%C0%9B%A8%FC%DC%FB%91%C1N%AE%8D%DBZ%E2b%F2%84%B6%E5&port=8888&uploaded=0&downloaded=0&left=555555&compact=1&event=started
        let http_tracker = HTTPTracker::new("rustorrent-client-dev", Client::new());
        let resp = http_tracker
            .get_announce_info(&mock_server.uri(), meta_info.info)
            .await;

        assert!(resp.is_ok());
    }
}
