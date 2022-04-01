use parking_lot::RwLock;
use std::sync::atomic::{AtomicU64, Ordering};

// The current state of the component
#[repr(u8)]
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum ComponentState {
    // Nothing happened to the component
    None = 0,

    // The component was added to it's current archetype
    Added = 1,

    // The component is pending for removal, and it will be removed next frame
    PendingForRemoval = 2,

    // THe component was mutated
    Mutated = 3,
}

// Stored component states
#[derive(Default)]
pub(crate) struct ComponentStatesBitfield {
    vec: RwLock<Vec<AtomicU64>>,
    length: usize,
}

impl ComponentStatesBitfield {
    // Reset all the bits to 0
    pub fn reset(&mut self) {
        for chunks in self.vec.get_mut().iter_mut() {
            *chunks.get_mut() = 0;
        }
    }
    // Set all the bits to the mutated state (0b11)
    pub fn set_all_mutated(&self) {
        for bundle in 0..self.length {
            self.set(bundle, ComponentState::Mutated);
        }
    }
    // Update the length of the sparse bitfield, and check if we must insert a new chunk
    pub fn set_len(&mut self, length: usize) {
        // Check if need to extend the sparse component mutation bitfields by one chunk
        let half = (u64::BITS as usize) / 2;
        let extend = length.div_ceil(half) > self.length.div_ceil(half);

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
    // Set the component state of a specific component
    pub fn set(&self, bundle: usize, state: ComponentState) {
        // Check if the index is valid
        dbg!(bundle);
        dbg!(self.length);
        dbg!(state);
        assert!(bundle < self.length, "Archetype bundle index is invalid");

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

        // (Only for components) Do not overwrite the state if it was already set
        if ((loaded >> local_pos) & 0b11) != 0 {
            return;
        }

        // 11 [00] 10 00
        let zeroed = loaded & !(0b11 << local_pos);

        // 00 11 00 00
        let state = unsafe { std::mem::transmute::<ComponentState, u8>(state) as u64 } << local_pos;

        // 11 11 10 00
        let result = zeroed | state;

        // Store
        atomic.store(result, Ordering::Relaxed);
    }
    // Iterate through the sparse set and return an Iterator full of component states
    pub fn iter(&self) -> impl IntoIterator<Item = ComponentState> {
        // We know the length, and each compoonent is already tightly packed
        let len = self.length;

        // Load all the chunks
        let read = self.vec.read();
        let chunks = read.iter().map(|atomic| atomic.load(Ordering::Relaxed)).collect::<Vec<u64>>();

        // Load each component state
        (0..len).into_iter().map(move |bundle| {
            // Get the local position, and chunk position
            let half = (u64::BITS as usize) / 2;
            let local_pos = bundle % half;
            let chunk_pos = bundle / half;

            // Load the bits from the chunk
            let bits = chunks.get(chunk_pos).unwrap();

            // Filter the specific bits
            let mask = 0b11 << local_pos;
            let filtered = bits & mask;
            let shifted = (filtered >> local_pos) as u8;
            unsafe { std::mem::transmute::<u8, ComponentState>(shifted) }
        })
    }
}
