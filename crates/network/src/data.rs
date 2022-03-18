use laminar::Packet;
use serde::{de::DeserializeOwned, Deserialize, Serialize};
use std::{
    cell::RefCell,
    collections::{hash_map::Entry, HashMap},
    io::{self, BufRead, BufReader, Cursor, Error, Read},
};
pub trait Payload: Serialize + DeserializeOwned + 'static {}
impl<T> Payload for T where T: Serialize + DeserializeOwned + 'static {}
pub type PayloadBucketId = u16;

// Serialize a payload, with it's packet bucket ID
pub fn serialize_payload<P: Payload>(bucket_id: PayloadBucketId, payload: P) -> Result<Vec<u8>, Error> {
    // Serialze the bucket ID
    let bucket_id_bytes = bucket_id.to_be_bytes();
    // Serialize the payload
    let payload = serde_json::to_string_pretty(&payload)?;

    // Convert to bytes
    let mut bucket_bytes = bucket_id_bytes.to_vec();
    let payload = payload.into_bytes();

    // Extend
    bucket_bytes.extend(payload);
    Ok(bucket_bytes)
}

// Desierliaze the payload bucket ID from received networked data
pub fn deserialize_bucket_id(buf: &[u8]) -> Result<PayloadBucketId, Error> {
    // Buffered
    let cursor = Cursor::new(buf);
    let mut reader = BufReader::new(cursor);

    // Deserialize the bucket ID
    let mut bucket_id_bytes = [0u8; 2];
    reader.read_exact(&mut bucket_id_bytes).unwrap();
    let bucket_id = PayloadBucketId::from_be_bytes(bucket_id_bytes);
    Ok(bucket_id)
}

// Deserialize a payload
pub fn deserialize_payload<P: Payload>(buf: &[u8]) -> Result<P, Error> {
    // Buffered
    let cursor = Cursor::new(buf);
    let mut reader = BufReader::new(cursor);

    // Ignore the first 2 bytes since they belong to the PayloadBucketId
    reader.seek_relative(2)?;

    // Then deserialize the payload
    let mut payload = Vec::default();
    reader.read_to_end(&mut payload)?;
    let payload = serde_json::from_slice::<P>(&payload)?;
    Ok(payload)
}
