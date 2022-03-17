use serde::{de::DeserializeOwned, Deserialize, Serialize};
use std::io::{self, BufRead, BufReader, Cursor, Read};

pub trait Payload: Serialize + DeserializeOwned {}
impl<T> Payload for T where T: Serialize + DeserializeOwned {}
// Packet metadata that contains some info on how we should treat the incoming payload
#[derive(Hash, PartialEq, Eq, Clone, Copy)]
pub struct PacketMetadata {
    // Payload type
    pub bucket_id: u64,
}

// Serialize a payload, with it's packet metadata
pub fn serialize_payload<P: Payload>(meta: PacketMetadata, payload: P) -> Result<Vec<u8>, io::Error> {
    // Serialze the metadata
    let meta = meta.bucket_id.to_be_bytes();
    // Serialize the payload
    let payload = serde_json::to_string_pretty(&payload)?;

    println!("{:?}", &meta);
    println!("{}", &payload);

    // Convert to bytes
    let mut meta = meta.to_vec();
    let payload = payload.into_bytes();

    // Extend
    meta.extend(payload);
    Ok(meta)
}