use laminar::Packet;
use serde::{de::DeserializeOwned, Deserialize, Serialize};
use std::{io::{self, BufRead, BufReader, Cursor, Read}, collections::{HashMap, hash_map::Entry}, cell::RefCell};


// Stored network cache
#[derive(Default)]
pub struct NetworkCache {
    // Buckets that contain multiple types of packet data (packet bucket id + payload)
    buckets: HashMap<PayloadBucketId, Vec<Packet>>,
}

impl NetworkCache {
    // Clear all cache
    pub fn clear(&mut self) {
        for (_, vec) in self.buckets.iter_mut() {
            vec.clear();
        }
    }
    // Drain a whole bucket of payloads
    pub fn drain_bucket(&mut self, bucket_id: PayloadBucketId) -> Option<Vec<Packet>> {
        let vec = self.buckets.get_mut(&bucket_id)?;
        Some(std::mem::take(vec))
    }
    // Push some received packet data into the corresponding bucket
    pub fn push(&mut self, packet: Packet) {
        // Buffered
        let cursor = Cursor::new(packet.payload());
        let mut reader = BufReader::new(cursor);

        // Deserialize the bucket ID
        let mut bucket_id_bytes = [0u8; 8];
        reader.read_exact(&mut bucket_id_bytes).unwrap();
        let bucket_id = PayloadBucketId::from_be_bytes(bucket_id_bytes);

        // Push the packet
        let vector = self.buckets.entry(bucket_id).or_default();
        vector.push(packet);
    }
}


pub trait Payload: Serialize + DeserializeOwned {}
impl<T> Payload for T where T: Serialize + DeserializeOwned {}
pub type PayloadBucketId = u64;

// Serialize a payload, with it's packet bucket ID
pub fn serialize_payload<P: Payload>(bucket_id: PayloadBucketId, payload: P) -> Result<Vec<u8>, io::Error> {
    // Serialze the bucket ID
    let bucket_id_bytes = bucket_id.to_be_bytes();
    // Serialize the payload
    let payload = serde_json::to_string_pretty(&payload)?;

    println!("{:?}", &bucket_id_bytes);
    println!("{}", &payload);

    // Convert to bytes
    let mut bucket_bytes = bucket_id_bytes.to_vec();
    let payload = payload.into_bytes();

    // Extend
    bucket_bytes.extend(payload);
    Ok(bucket_bytes)
}

// Deserialize a payload
pub fn deserialize_payload<P: Payload>(buf: Vec<u8>) -> Result<(PayloadBucketId, P), io::Error> {
    // Buffered
    let cursor = Cursor::new(&buf);
    let mut reader = BufReader::new(cursor);

    // Deserialize the bucket ID first
    let mut bucket_id_bytes = [0u8; 8];
    reader.read_exact(&mut bucket_id_bytes)?;
    let bucket_id = PayloadBucketId::from_be_bytes(bucket_id_bytes);

    // Then deserialize the payload
    let mut payload = Vec::default();
    reader.read_to_end(&mut payload)?;
    let payload = serde_json::from_slice::<P>(&payload)?;
    Ok((bucket_id, payload))
}