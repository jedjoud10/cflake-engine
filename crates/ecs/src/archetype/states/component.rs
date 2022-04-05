use parking_lot::RwLock;
use std::sync::atomic::{AtomicU64, AtomicUsize, Ordering};

// Stored component mutation states
#[derive(Default)]
pub(crate) struct ComponentMutationsBitfield {
    vec: RwLock<Vec<AtomicU64>>,
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
    // Bunde index into local pos and chunk pos
    const fn to_indices(bundle: usize) -> (usize, usize) {
        // Get the local position and chunk position
        const FULL: usize = (u64::BITS as usize);
        let local = bundle % FULL;
        let chunk = bundle / FULL;
        return (local, chunk)
    }
    // Check if a component was mutated
    pub fn get(&self, bundle: usize) -> bool {
        // Be ready to read from the vector
        let (local_pos, chunk_pos) = Self::to_indices(bundle);
        let read = self.vec.read();
        let bits = read.get(chunk_pos as usize).unwrap().load(Ordering::Relaxed);

        // Check if it was mutated
        (bits >> local_pos) % 2 == 1
    }
    // Set the mutation state of a specific component
    pub fn set(&self, bundle: usize) {
        // Be ready to read from the vector
        let (local_pos, chunk_pos) = Self::to_indices(bundle);
        let vec = self.vec.read();
        let atomic = vec.get(chunk_pos as usize).unwrap();

        // Write to the vector
        atomic.fetch_or(1 << local_pos, Ordering::Relaxed);
    }
    // Iterate through the stored component states
    pub fn iter(&self) -> Iter {
        Iter { states: self, len: self.vec.read().len(), bundle: 0, loaded: None }
    }
}


// Custom iterator
pub(crate) struct Iter<'a> {
    // The main bitfield
    states: &'a ComponentMutationsBitfield,
    len: usize,

    // Iteration values
    bundle: usize,
    loaded: Option<u64>,
}

impl<'a> Iterator for Iter<'a> {
    type Item = bool;

    fn next(&mut self) -> Option<Self::Item> {
        // Get the local position and chunk position
        let (local_pos, chunk_pos) = ComponentMutationsBitfield::to_indices(self.bundle);

        // Clear the loaded chunk
        if local_pos == 63 { self.loaded.take()?; }

        // Invalid chunk pos
        if chunk_pos >= self.len { return None }

        // Try to load a chunk into memory
        self.loaded.get_or_insert_with(|| {
            let chunks = self.states.vec.read();
            chunks.get(chunk_pos).unwrap().load(Ordering::Relaxed)
        });

        // Read the mutation bit
        let filtered = (self.loaded.unwrap() >> local_pos) % 2 == 1;
        Some(filtered)
    }
}