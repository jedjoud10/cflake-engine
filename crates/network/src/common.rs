use std::io::{self, BufRead, BufReader, Cursor};

use serde::{de::DeserializeOwned, Deserialize, Serialize};

// Packet metadata that contains some info on how we should treat the incoming payload
#[derive(Serialize, Deserialize)]
pub struct PacketMetadata {
    // Payload type
    pub id: u64,
}

// Serialize a payload, with it's packet metadata
pub fn serialize_payload<Payload: Serialize>(meta: PacketMetadata, payload: Payload) -> Result<Vec<u8>, io::Error> {
    // Serialze the metadata
    let meta = serde_json::to_string_pretty(&meta)?;
    // Serialize the payload
    let payload = serde_json::to_string_pretty(&payload)?;

    println!("{}", &meta);
    println!("{}", &payload);

    // Convert to bytes
    let mut meta = meta.into_bytes();
    let payload = payload.into_bytes();

    // Extend
    meta.push(0);
    meta.extend(payload);
    meta.push(0);
    Ok(meta)
}

// Deserialize a packet, and write it to a buffer
pub fn deserialize_payload<Payload: DeserializeOwned>(buf: &[u8], id: u64) -> Result<(PacketMetadata, Payload), io::Error> {
    // Buf reader
    let cursor = Cursor::new(buf);
    let mut reader = BufReader::new(cursor);
    // Split at the end of the metadata
    let mut metadata = Vec::default();
    let mut payload = Vec::default();
    reader.read_until(0, &mut metadata)?;
    reader.read_until(0, &mut payload)?;
    metadata.pop();
    payload.pop();
    // Deserialize
    let metadata = serde_json::from_slice::<PacketMetadata>(&metadata)?;
    assert!(metadata.id == id, "Metadata ID does not match up with PacketReceiver's ID");
    let payload = serde_json::from_slice::<Payload>(&payload)?;
    Ok((metadata, payload))
}
