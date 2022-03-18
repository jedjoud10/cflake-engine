use std::{
    any::TypeId,
    sync::atomic::{AtomicU16, AtomicU32, AtomicU64, Ordering},
};

use ahash::AHashMap;
use lazy_static::lazy_static;
use parking_lot::RwLock;

use crate::{Payload, PayloadBucketId};

// Use to keep track of the payload bucket IDs
lazy_static! {
    static ref NEXT_REGISTERED_BUCKET_ID: AtomicU16 = AtomicU16::new(0);
    static ref REGSISTERED_BUCKET_IDS: RwLock<AHashMap<TypeId, PayloadBucketId>> = RwLock::new(AHashMap::new());
}

// Register a type using a new bucket ID
pub fn register<P: Payload + 'static>() -> PayloadBucketId {
    // Register the type
    let mut lock = REGSISTERED_BUCKET_IDS.write();
    // Add the current ID in the hashmap, then increment it
    let previous = NEXT_REGISTERED_BUCKET_ID.fetch_add(1, Ordering::Relaxed);
    lock.insert(TypeId::of::<P>(), previous);
    previous
}

// Get the bucket ID of a specific payload type
pub fn get_bucket_id<P: Payload + 'static>() -> Option<PayloadBucketId> {
    // Read
    let read = REGSISTERED_BUCKET_IDS.read();
    read.get(&TypeId::of::<P>()).cloned()
}
