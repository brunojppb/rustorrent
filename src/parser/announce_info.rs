use crate::parser::bencode::{Bencode, BencodeError};
use crate::parser::byte_string::ByteString;

/// Response from announce tracker servers
#[derive(Debug, Clone)]
pub struct AnnounceInfo {
    pub interval: u64,
    pub complete: u64,
    pub incomplete: u64,
    pub peers: Vec<Peer>,
    pub min_interval: Option<u64>,
    pub tracker_id: Option<String>,
}

impl AnnounceInfo {
    pub fn parse(value: &Bencode) -> Result<Self, BencodeError> {
        let err = |msg: &str| -> Result<Self, BencodeError> {
            Err(BencodeError::new(format!(
                "Invalid bencode value for AnounceInfo when decoding \"{}\": {:?}",
                msg, value
            )))
        };

        let Bencode::Dict(map) = value else {
            return err("initial value");
        };

        let Some(Bencode::Number(complete)) = map.get(&ByteString::new("complete"))  else {
            return err("complete");
        };

        let Some(Bencode::Number(incomplete)) = map.get(&ByteString::new("incomplete")) else {
            return err("incomplete");
        };

        let Some(Bencode::Number(interval)) = map.get(&ByteString::new("interval")) else {
            return err("interval");
        };

        let Some(Bencode::List(peers_list)) =
                                map.get(&ByteString::new("peers")) else {
            return err("peers");
        };

        let maybe_tracker_id = map
            .get(&ByteString::new("tracker id"))
            .and_then(|v| match v {
                Bencode::Text(peer_id) => Some(peer_id.to_string()),
                _ => None,
            });
        let mut peers = Vec::with_capacity(peers_list.len());
        for peer_dict in peers_list.iter() {
            let peer = Peer::parse(peer_dict)?;
            peers.push(peer);
        }

        Ok(Self {
            complete: complete.to_owned(),
            incomplete: incomplete.to_owned(),
            interval: interval.to_owned(),
            peers,
            tracker_id: maybe_tracker_id,
            min_interval: None,
        })
    }
}

#[derive(Debug, Clone)]
pub struct Peer {
    pub peer_id: String,
    pub ip: String,
    pub port: u64,
}

impl Peer {
    // TODO: Must handle peers in the binary model format as well.
    // The peers value may be a string consisting of multiples of 6 bytes.
    // First 4 bytes are the IP address and last 2 bytes are the port number,
    // all in network (big endian) notation.
    pub fn parse(value: &Bencode) -> Result<Self, BencodeError> {
        let err = |msg: &str| -> Result<Self, BencodeError> {
            Err(BencodeError::new(format!(
                "Invalid bencode value for peer when decoding \"{}\": {:?}",
                msg, value
            )))
        };
        let Bencode::Dict(map) = value else {
            return err("raw value");
        };

        let Some(Bencode::Text(peer_id)) = map.get(&ByteString::new("peer id")) else {
            return err("peer id");
        };

        let Some(Bencode::Text(ip)) = map.get(&ByteString::new("ip")) else {
            return err("ip");
        };

        let Some(Bencode::Number(port)) = map.get(&ByteString::new("port"))  else {
            return err("port");
        };

        Ok(Self {
            peer_id: peer_id.to_string(),
            ip: ip.to_string(),
            port: port.to_owned(),
        })
    }
}
