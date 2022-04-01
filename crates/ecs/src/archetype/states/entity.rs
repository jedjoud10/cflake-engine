use parking_lot::RwLock;
use std::sync::atomic::{AtomicU64, Ordering, AtomicUsize};

// The current state of the bundle entity
#[repr(u8)]
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum EntityState {
    // Nothing happened to the entity
    None = 0,

    // The entity just got added to the archetype
    Added = 1,

    // The bundled entity is pending for removal, and it will be removed next frame
    PendingForRemoval = 2,
}

// Stored entity states
#[derive(Default)]
pub(crate) struct EntityStatesBitfield {
    vec: RwLock<Vec<AtomicU64>>,
    length: AtomicUsize,
}

impl EntityStatesBitfield {
    // Reset all the bits to 0
    pub fn reset(&mut self) {
        for chunks in self.vec.get_mut().iter_mut() {
            *chunks.get_mut() = 0;
        }
    }
    // Extend by one chunk
    pub fn extend(&self) {
        self.vec.write().push(AtomicU64::new(0));
    }
    // Check if a coponent was mutated
    pub fn get(&self, bundle: usize) -> EntityState {
        // Get the local position, and chunk position
        let half = (u64::BITS as usize) / 2;
        let local_pos = bundle % half;
        let chunk_pos = bundle / half;

        // Be ready to read from the vector
        let vec = self.vec.read();
        let atomic = vec.get(chunk_pos as usize).unwrap();

        // Load the bits from the atomic
        // 11 [01] 10 00
        let loaded = atomic.load(Ordering::Relaxed);

        // 00 00 11 [01]
        let filtered = (loaded >> local_pos) & 0b11;
        unsafe { std::mem::transmute::<u8, EntityState>(filtered as u8) }
    }
    // Set the state of a specific bundle entity
    pub fn set(&self, bundle: usize, state: EntityState) {
        // Get the local position, and chunk position
        let half = (u64::BITS as usize) / 2;
        let local_pos = bundle % half;
        let chunk_pos = bundle / half;

        // Be ready to write to the vector
        let vec = self.vec.read();
        let atomic = vec.get(chunk_pos as usize).unwrap();

        // Load the bits from the atomic
        // 11 [01] 10 00
        let loaded = atomic.load(Ordering::Relaxed);

        // 11 [00] 10 00
        let zeroed = loaded & !(0b11 << local_pos);

        // 00 11 00 00
        let state = unsafe { std::mem::transmute::<EntityState, u8>(state) as u64 } << local_pos;

        // 11 11 10 00
        let result = zeroed | state;

        // Store
        atomic.store(result, Ordering::Relaxed);
    }
}
