use serde::{de::DeserializeOwned, Deserialize, Serialize};
use std::io::{self, BufRead, BufReader, Cursor, Read};

pub trait Payload: Serialize + DeserializeOwned {}
impl<T> Payload for T where T: Serialize + DeserializeOwned {}
// Packet metadata that contains some info on how we should treat the incoming payload
pub struct PacketMetadata {
    // Payload type
    pub id: u64,
}

// Serialize a payload, with it's packet metadata
pub fn serialize_payload<P: Payload>(meta: PacketMetadata, payload: P) -> Result<Vec<u8>, io::Error> {
    // Serialze the metadata
    let meta = meta.id.to_be_bytes();
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

// Deserialize a packet, and write it to a buffer
pub fn deserialize_payload<P: Payload>(buf: &[u8], id: u64) -> Result<(PacketMetadata, P), io::Error> {
    // Buf reader
    let cursor = Cursor::new(buf);
    let mut reader = BufReader::new(cursor);
    // Split at the end of the metadata
    let mut metadata = [0u8; 8];
    let mut payload = Vec::default();
    
    // Read the u64 id data
    reader.read_exact(&mut metadata)?;
    reader.read_to_end(&mut payload)?;
    // Deserialize
    let metadata = PacketMetadata { id: u64::from_be_bytes(metadata) };
    assert!(metadata.id == id, "Metadata ID does not match up with PacketReceiver's ID");
    let payload = serde_json::from_slice::<P>(&payload)?;
    Ok((metadata, payload))
}
