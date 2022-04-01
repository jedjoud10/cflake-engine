use parking_lot::RwLock;
use std::sync::atomic::{AtomicU64, Ordering};

// Stored component mutation states
#[derive(Default)]
pub(crate) struct ComponentMutationsBitfield {
    vec: RwLock<Vec<AtomicU64>>,
    length: usize,
}

impl ComponentMutationsBitfield {
    // Set all the mutation states to be true
    pub fn set_all_mutated(&self) {
        for chunks in self.vec.read().iter() {
            chunks.store(u64::MAX, Ordering::Relaxed);
        }
    }
    // Reset all the bits to 0
    pub fn reset(&mut self) {
        for chunks in self.vec.get_mut().iter_mut() {
            *chunks.get_mut() = 0;
        }
    }
    // Update the length of the sparse bitfield, and check if we must insert a new chunk
    pub fn set_len(&mut self, length: usize) {
        // Check if need to extend the sparse bitfield by one chunk
        let extend = length.div_ceil(u64::BITS as usize) > self.length.div_ceil(u64::BITS as usize);

        // Extend by one chunk
        if extend {
            self.vec.get_mut().push(AtomicU64::new(0));
        }

        // Set length
        self.length = length;
    }
    /*
    // Get the state of a specific component in the archetype
    pub fn get(&self, bundle: usize) -> ComponentState {
        // Check if the index is valid
        assert!(bundle < self.length, "Archetype bundle index is invalid");

        // Get the local position, and chunk position
        let half = (u64::BITS as usize) / 2;
        let local_pos = bundle % half;
        let chunk_pos = bundle / half;

        // Read from the vector now
        let read = self.vec.read();
        let bits = read.get(chunk_pos as usize).unwrap().load(Ordering::Relaxed);

        // Filter the specific bits
        let mask = 0b11 << local_pos;
        let filtered = bits & mask;
        let shifted = (filtered >> local_pos) as u8;
        unsafe { std::mem::transmute::<u8, ComponentState>(shifted) }
    }
    */
    // Set the mutation state of a specific component
    pub fn set_mutated_state(&self, bundle: usize) {
        // Check if the index is valid
        assert!(bundle < self.length, "Bundle index is invalid");

        // Get the local position, and chunk position
        let local_pos = bundle % u64::BITS as usize;
        let chunk_pos = bundle / u64::BITS as usize;

        // Be ready to write to the vector
        let vec = self.vec.read();
        let atomic = vec.get(chunk_pos as usize).unwrap();

        // Write to the vector
        atomic.fetch_or(1 << local_pos, Ordering::Relaxed);
    }
    // Iterate through the sparse set and return an Iterator full of mutation states (booleans)
    pub fn iter(&self) -> impl IntoIterator<Item = bool> {
        // We know the length, and each component is already tightly packed
        let len = self.length;

        // Load all the chunks
        let read = self.vec.read();
        let chunks = read.iter().map(|atomic| atomic.load(Ordering::Relaxed)).collect::<Vec<u64>>();

        // Load each component state
        (0..len).into_iter().map(move |bundle| {
            // Get the local position, and chunk position
            let local_pos = bundle % (u64::BITS as usize);
            let chunk_pos = bundle / (u64::BITS as usize);

            // Load the bits from the chunk
            let bits = chunks.get(chunk_pos).unwrap();

            // Check if it was mutated
            let was_mutated = (bits >> local_pos) % 2 == 1;
            was_mutated
        })
    }
}
