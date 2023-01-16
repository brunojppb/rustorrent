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
        match value {
            Bencode::Dict(map) => {
                if let Some(Bencode::Number(complete)) = map.get(&ByteString::new("complete")) {
                    if let Some(Bencode::Number(incomplete)) =
                        map.get(&ByteString::new("incomplete"))
                    {
                        if let Some(Bencode::Number(interval)) =
                            map.get(&ByteString::new("interval"))
                        {
                            if let Some(Bencode::List(peers_list)) =
                                map.get(&ByteString::new("peers"))
                            {
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
                                return Ok(Self {
                                    complete: complete.to_owned(),
                                    incomplete: incomplete.to_owned(),
                                    interval: interval.to_owned(),
                                    peers,
                                    tracker_id: maybe_tracker_id,
                                    min_interval: None,
                                });
                            }
                        }
                    }
                }

                Err(BencodeError::new(format!(
                    "Invalid bencode value for announce info: {:?}",
                    value
                )))
            }
            _ => Err(BencodeError::new(format!(
                "Invalid bencode value for announce info: {:?}",
                value
            ))),
        }
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
        match value {
            Bencode::Dict(map) => {
                if let Some(Bencode::Text(peer_id)) = map.get(&ByteString::new("peer id")) {
                    if let Some(Bencode::Text(ip)) = map.get(&ByteString::new("ip")) {
                        if let Some(Bencode::Number(port)) = map.get(&ByteString::new("port")) {
                            return Ok(Self {
                                peer_id: peer_id.to_string(),
                                ip: ip.to_string(),
                                port: port.to_owned(),
                            });
                        }
                    }
                }
                Err(BencodeError::new(format!(
                    "Invalid bencode value for peer: {:?}",
                    value
                )))
            }
            _ => Err(BencodeError::new(format!(
                "Invalid bencode value for peer: {:?}",
                value
            ))),
        }
    }
}
