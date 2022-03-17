use std::{collections::HashMap, cell::RefCell, borrow::BorrowMut};

use crate::{PacketMetadata};

// Stored network cache
pub struct NetworkCache {
    buckets: HashMap<PacketMetadata, RefCell<Vec<Vec<u8>>>>,
    pub(crate) max_buffer_size: usize,
}

impl NetworkCache {
    pub fn new(max_buffer_size: usize) -> Self {
        Self {
            buckets: Default::default(),
            max_buffer_size,
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
    pub fn drain_bucket(&self, meta: PacketMetadata) -> Option<Vec<Vec<u8>>> {
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
    pub fn push(&mut self, meta: PacketMetadata, data: Vec<u8>) {

    }
}