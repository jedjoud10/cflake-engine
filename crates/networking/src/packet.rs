use std::{
    any::TypeId,
    collections::hash_map::DefaultHasher,
    hash::{Hash, Hasher},
};

// This is a network packet that we can send to clients / to the server
// Packets must be serializable and must solely represent plain old data
pub trait Packet: 'static + Clone + Copy + serde::de::DeserializeOwned + serde::Serialize {}
impl<T> Packet for T where T: 'static + Clone + Copy + serde::de::DeserializeOwned + serde::Serialize
{}

// Get the unique id for a specific packet type
// TODO: Don't fucking use type ID and this shit
pub fn id<T: Packet>() -> u64 {
    let id = TypeId::of::<T>();
    let mut hasher = DefaultHasher::default();
    id.hash(&mut hasher);
    hasher.finish()
}
