use std::io::{Write, BufWriter};

use serde::{Serialize, de::{DeserializeOwned, Visitor}, Deserialize};


// A packet that contains some payload and info from where it comes from
#[derive(Serialize, Deserialize)]
pub struct Packet<Payload> {
    // From which client this packet is from
    pub client_id: u32,

    // Actual payload data
    pub payload: Payload,
}

// Serialize a packet, and write it to a stream
pub fn write<Payload: Serialize, W: Write>(packet: Packet<Payload>, stream: &mut W) {
    // Write
    let data = serde_json::to_vec_pretty(&packet).unwrap();
    stream.write(&data).unwrap();
}

// Deserialize a packet, and write it to a buffer
pub fn read<'a, Payload: Deserialize<'a>>(data: &'a [u8], output: &mut Vec<Packet<Payload>>) {
    // Read
    let packet = serde_json::from_slice::<'a, Packet<Payload>>(data).unwrap();
    output.push(packet);
}