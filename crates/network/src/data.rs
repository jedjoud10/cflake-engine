use serde::{de::DeserializeOwned, Deserialize, Serialize};
use std::{io::{self, BufRead, BufReader, Cursor, Read}, collections::HashMap, cell::RefCell};


// Stored network cache
#[derive(Default)]
pub struct NetworkCache {
    buckets: HashMap<PacketBucketId, RefCell<Vec<Vec<u8>>>>,
}

impl NetworkCache {
    pub fn new(max_buffer_size: usize) -> Self {
        Self {
            buckets: Default::default(),
        }
    }
    // Clear all cache
    pub fn clear(&mut self) {
        for (_, slots) in self.buckets.iter() {
            let mut borrow = slots.borrow_mut();
            borrow.clear();
        }
    }
    // Drain a whole bucket of payloads
    pub fn drain_bucket(&self, meta: PacketBucketId) -> Option<Vec<Vec<u8>>> {
        let vec = self.buckets.get(&meta)?;
        let mut borrowed = vec.borrow_mut();
        let stolen = std::mem::take(&mut *borrowed);
        if stolen.is_empty() {
            None
        } else {
            Some(stolen)
        }
    }
    // Push some received payload data into the corresponding slot
    pub fn push(&mut self, meta: PacketBucketId, data: Vec<u8>) {

    }
}


pub trait Payload: Serialize + DeserializeOwned {}
impl<T> Payload for T where T: Serialize + DeserializeOwned {}
pub type PacketBucketId = u64;

// Serialize a payload, with it's packet bucket ID
pub fn serialize_payload<P: Payload>(bucket_id: PacketBucketId, payload: P) -> Result<Vec<u8>, io::Error> {
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
pub fn deserialize_payload<P: Payload>(buf: Vec<u8>) -> Result<(PacketBucketId, P), io::Error> {
    // Buffered
    let cursor = Cursor::new(&buf);
    let mut reader = BufReader::new(cursor);

    // Deserialize the bucket ID first
    let mut bucket_id_bytes = [0u8; 8];
    reader.read_exact(&mut bucket_id_bytes)?;
    let bucket_id = PacketBucketId::from_be_bytes(bucket_id_bytes);

    // Then deserialize the payload
    let mut payload = Vec::default();
    reader.read_to_end(&mut payload)?;
    let payload = serde_json::from_slice::<P>(&payload)?;
    Ok((bucket_id, payload))
}