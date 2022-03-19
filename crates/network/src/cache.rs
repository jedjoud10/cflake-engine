use std::{collections::HashMap, cell::RefCell};

use laminar::Packet;

use crate::{deserialize_bucket_id, deserialize_payload, registry, Payload, PayloadBucketId};

// Stored network cache
#[derive(Default)]
pub struct NetworkCache {
    // Buckets that contain multiple types of packet data (packet bucket id + payload)
    buckets: RefCell<HashMap<PayloadBucketId, Vec<Packet>>>,
}

impl NetworkCache {
    // Clear all cache
    pub fn clear(&self) {
        let mut buckets = self.buckets.borrow_mut();
        for (_, vec) in buckets.iter_mut() {
            vec.clear();
        }
    }
    // Drain a whole bucket of packets into a payload cache
    pub fn drain_bucket_to_payload_cache<P: Payload>(&self, cache: &mut PayloadCache<P>) {
        let mut buckets = self.buckets.borrow_mut();
        let bucket_id = registry::get_bucket_id::<P>();
        let vec = buckets.entry(bucket_id).or_default();
        let vec = std::mem::take(vec);
        // Deserialize
        let payloads = vec.into_iter().map(|packet| deserialize_payload(packet.payload()).unwrap()).collect::<Vec<P>>();
        cache.payloads = payloads;
    }
    // Push some received packet data into the corresponding bucket
    pub(crate) fn push(&self, packet: Packet) -> u16 {
        let mut buckets = self.buckets.borrow_mut();
        let bucket_id = deserialize_bucket_id(packet.payload()).unwrap();
        // Push the packet
        let vector = buckets.entry(bucket_id).or_default();
        vector.push(packet);
        bucket_id
    }
}

// Payload cache
pub struct PayloadCache<P: Payload> {
    payloads: Vec<P>,
}

impl<P: Payload> Default for PayloadCache<P> {
    fn default() -> Self {
        Self { payloads: Default::default() }
    }
}

impl<P: Payload> PayloadCache<P> {
    // Iter
    pub fn iter(&self) -> impl Iterator<Item = &P> {
        self.payloads.iter()
    }
    // Get newest payload
    pub fn newest(&self) -> Option<&P> {
        self.payloads.first()
    }
    // Get oldest payload
    pub fn oldest(&self) -> Option<&P> {
        self.payloads.last()
    }
}
