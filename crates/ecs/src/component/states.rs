use parking_lot::RwLock;
use std::mem::size_of;

// Common values
pub const MUTATED_STATE: u8 = 1;
pub const ADDED_STATE: u8 = 2;
pub const REMOVED_STATE: u8 = 3;

// The bitflags for each component, stored in each component storage in each archetype
#[derive(Default)]
pub struct SparseComponentStates {
    // Each bitflags contains 4 states (2 bits per component), (nil, added, removed, changed)
    vec: RwLock<Vec<u64>>,

    // The amount of bundles (component elements) that we have
    count: usize,
}

impl SparseComponentStates {
    // Reset all the flags to 0
    pub fn reset(&mut self) {
        for chunks in self.vec.get_mut().iter_mut() {
            *chunks = 0
        }
    }
    // Extend the component flags by one element (basically adds a component)
    pub fn extend_by_one(&mut self) {
        // Incremement
        self.count += 1;

        // Increase the number of chunks if needed
        let new_len = ((self.count * 2) / (size_of::<u64>() * 8)) + 1;
        if new_len > self.vec.get_mut().len() {
            self.vec.get_mut().resize(new_len, 0);
        }
    }

    // Get the bitflags for a specific component
    pub fn get(&self, bundle: usize) -> u8 {
        // 00 00 00 00   00 00 11 00
        // Get the local position, and chunk position
        let bundle = bundle * 2;
        let local_pos = bundle % size_of::<u64>();
        let chunk_pos = bundle / (size_of::<u64>() * 8);

        // Read from the vector now
        let read = self.vec.read();
        let bits = read.get(chunk_pos as usize).unwrap();

        // Filter the specific bits
        let mask = 1 << local_pos | 2 << local_pos;
        let filtered = bits & mask;
        (filtered >> local_pos) as u8
    }
    // Set the bitflags for a specific component
    pub fn set(&self, bundle: usize, flags: u8) {
        // The flags parameter should only contain two bits set
        assert!(flags < 4, "Flags not valid!");

        let bundle = bundle * 2;
        let local_pos = bundle % size_of::<u64>();
        let chunk_pos = bundle / (size_of::<u64>() * 8);

        // Be ready to write to the vector
        let mut vec = self.vec.write();
        let bits = vec.get_mut(chunk_pos as usize).unwrap();

        // This will not overwrite if the state has already been set
        if (*bits >> local_pos) & 3 == 0 {
            // Filter the specific bits
            let mask = (flags as u64) << local_pos;
            *bits |= mask;
        }
    }
}
