use crossbeam_channel::Sender;
use laminar::Packet;
use serde::{de::DeserializeOwned, Serialize};
use std::{
    io::{BufReader, Cursor, Error, Read},
    net::SocketAddr,
};

use crate::registry;
pub trait Payload: Serialize + DeserializeOwned + 'static {}
impl<T> Payload for T where T: Serialize + DeserializeOwned + 'static {}
pub type PayloadBucketId = u16;

// Serialize a payload, with it's packet bucket ID
pub fn serialize_payload<P: Payload>(
    bucket_id: PayloadBucketId,
    payload: P,
) -> Result<Vec<u8>, Error> {
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

// The type of packets that we send (reliability / order)
pub enum PacketType {
    UnreliableUnordered,
    ReliableUnordered,
    ReliableOrdered,
    ReliableSequenced,
    UnreliableSequenced,
}

// Helper functions that automatically serialize the payload before sending it
pub fn send<P: Payload + 'static>(
    recv: SocketAddr,
    payload: P,
    sender: &Sender<Packet>,
    _type: PacketType,
) -> Result<(), Error> {
    match _type {
        PacketType::UnreliableUnordered => send_unreliable_unordered(recv, payload, sender),
        PacketType::ReliableUnordered => send_reliable_unordered(recv, payload, sender),
        PacketType::ReliableOrdered => send_reliable_ordered(recv, payload, sender),
        PacketType::ReliableSequenced => send_reliable_sequenced(recv, payload, sender),
        PacketType::UnreliableSequenced => send_unreliable_sequenced(recv, payload, sender),
    }
}
fn send_unreliable_unordered<P: Payload + 'static>(
    recv: SocketAddr,
    payload: P,
    sender: &Sender<Packet>,
) -> Result<(), Error> {
    let bucket_id = registry::get_bucket_id::<P>();
    let packet = Packet::unreliable(recv, serialize_payload(bucket_id, payload)?);
    sender.send(packet).unwrap();
    Ok(())
}
fn send_reliable_unordered<P: Payload + 'static>(
    recv: SocketAddr,
    payload: P,
    sender: &Sender<Packet>,
) -> Result<(), Error> {
    let bucket_id = registry::get_bucket_id::<P>();
    let packet = Packet::reliable_unordered(recv, serialize_payload(bucket_id, payload)?);
    sender.send(packet).unwrap();
    Ok(())
}
fn send_reliable_ordered<P: Payload + 'static>(
    recv: SocketAddr,
    payload: P,
    sender: &Sender<Packet>,
) -> Result<(), Error> {
    let bucket_id = registry::get_bucket_id::<P>();
    let packet = Packet::reliable_ordered(
        recv,
        serialize_payload(bucket_id, payload)?,
        Some(bucket_id.try_into().unwrap()),
    );
    sender.send(packet).unwrap();
    Ok(())
}
fn send_reliable_sequenced<P: Payload + 'static>(
    recv: SocketAddr,
    payload: P,
    sender: &Sender<Packet>,
) -> Result<(), Error> {
    let bucket_id = registry::get_bucket_id::<P>();
    let packet = Packet::reliable_sequenced(
        recv,
        serialize_payload(bucket_id, payload)?,
        Some(bucket_id.try_into().unwrap()),
    );
    sender.send(packet).unwrap();
    Ok(())
}
fn send_unreliable_sequenced<P: Payload + 'static>(
    recv: SocketAddr,
    payload: P,
    sender: &Sender<Packet>,
) -> Result<(), Error> {
    let bucket_id = registry::get_bucket_id::<P>();
    let packet = Packet::unreliable_sequenced(
        recv,
        serialize_payload(bucket_id, payload)?,
        Some(bucket_id.try_into().unwrap()),
    );
    sender.send(packet).unwrap();
    Ok(())
}
