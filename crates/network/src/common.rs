use std::{io::{Write, BufWriter}, any::TypeId};

use serde::{Serialize, de::{DeserializeOwned, Visitor}, Deserialize};


// Packet metadata that contains some info on how we should treat the incoming payload
#[derive(Serialize, Deserialize)]
pub struct PacketMetadata {
    // Payload type
    pub id: u64,
}

// Serialize a payload, with it's packet metadata
pub fn serialize_payload<Payload: Serialize>(meta: PacketMetadata, payload: Payload) -> Vec<u8> {
    // Serialze the metadata
    let meta = serde_json::to_string_pretty(&meta).unwrap();
    // Serialize the payload
    let payload = serde_json::to_string_pretty(&payload).unwrap();

    println!("{}", &meta);
    println!("{}", &payload);

    // Convert to bytes
    let mut meta = meta.into_bytes();
    let payload = payload.into_bytes();
    
    // Extend
    meta.push(0);
    meta.extend(payload);
    meta.push(0);
    meta
}

// Deserialize a packet, and write it to a buffer
pub fn deserialize_payload<'a, Payload: Deserialize<'a>>(data: &'a [u8], output: &mut Vec<(PacketMetadata, Payload)>) {
    /*
    // Read
    let packet = serde_json::from_slice::<'a, Payload>(data).unwrap();
    output.push(packet);
    */
}