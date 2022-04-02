use parking_lot::RwLock;
use std::sync::atomic::{AtomicU64, Ordering, AtomicUsize};

// Stored component mutation states
#[derive(Default)]
pub(crate) struct ComponentMutationsBitfield {
    vec: RwLock<Vec<AtomicU64>>,
    length: AtomicUsize,
}

impl ComponentMutationsBitfield {
    // Reset all the bits to a specific state
    pub fn reset_to(&self, state: bool) {
        // If state is true, all the bits are set. If it is false, none of the bits are set
        let bits = if state { u64::MAX } else { 0 };
        for chunks in self.vec.read().iter() {
            chunks.store(bits, Ordering::Relaxed);
        }
    }
    // Extend by one chunk
    pub fn extend(&self) {
        self.vec.write().push(AtomicU64::new(0));
    }
    // Check if a component was mutated
    pub fn get(&self, bundle: usize) -> bool {
        // Get the local position, and chunk position
        let local_pos = bundle % (u64::BITS as usize);
        let chunk_pos = bundle / (u64::BITS as usize);

        // Read from the vector now
        let read = self.vec.read();
        let bits = read.get(chunk_pos as usize).unwrap().load(Ordering::Relaxed);

        // Check if it was mutated
        (bits >> local_pos) % 2 == 1
    }
    // Set the mutation state of a specific component
    pub fn set(&self, bundle: usize) {
        // Get the local position, and chunk position
        let local_pos = bundle % u64::BITS as usize;
        let chunk_pos = bundle / u64::BITS as usize;

        // Be ready to write to the vector
        let vec = self.vec.read();
        let atomic = vec.get(chunk_pos as usize).unwrap();

        // Write to the vector
        atomic.fetch_or(1 << local_pos, Ordering::Relaxed);
    }
}
