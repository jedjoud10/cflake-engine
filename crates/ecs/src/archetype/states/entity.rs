use parking_lot::RwLock;
use std::sync::atomic::{AtomicU64, AtomicUsize, Ordering};

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
}

impl EntityStatesBitfield {
    // Reset all the bits to a specific state
    pub fn reset_to(&self, state: EntityState) {
        // I don't like this
        let pattern = match state {
            EntityState::None => 0,
            EntityState::Added => 0x5555555555555555,
            EntityState::PendingForRemoval => u64::MAX,
        };
        for chunks in self.vec.read().iter() {
            chunks.store(pattern, Ordering::Relaxed);
        }
    }
    // Extend by one chunk
    pub fn extend(&self) {
        self.vec.write().push(AtomicU64::new(0));
    }
    // Check if a coponent was mutated
    pub fn get(&self, bundle: usize) -> EntityState {
        // Get the local position and chunk position
        let half = (u64::BITS as usize) / 2;
        let local_pos = bundle % half;
        let chunk_pos = bundle / half;

        // Be ready to read from the vector
        let vec = self.vec.read();
        let atomic = vec.get(chunk_pos as usize).unwrap();

        // Load the bits from the atomic
        let loaded = atomic.load(Ordering::Relaxed);
        let filtered = (loaded >> local_pos) & 0b11;
        unsafe { std::mem::transmute::<u8, EntityState>(filtered as u8) }
    }
    // Set the state of a specific bundle entity
    pub fn set(&self, bundle: usize, state: EntityState) {
        // Get the local position and chunk position
        let half = (u64::BITS as usize) / 2;
        let local_pos = bundle % half;
        let chunk_pos = bundle / half;

        // Be ready to write to the vector
        let vec = self.vec.read();
        let atomic = vec.get(chunk_pos as usize).unwrap();

        // Load the bits from the atomic
        let loaded = atomic.load(Ordering::Relaxed);
        let zeroed = loaded & !(0b11 << local_pos);
        let state = unsafe { std::mem::transmute::<EntityState, u8>(state) as u64 } << local_pos;
        let result = zeroed | state;
        atomic.store(result, Ordering::Relaxed);
    }
}
