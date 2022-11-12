use itertools::Itertools;
use smallvec::SmallVec;

// Simple bitset that allocates using u64 chunks
// This bitset contains a specific number of elements per chunk
pub struct BitSet(Vec<usize>, bool);

impl BitSet {
    // Create a new empty bit set
    pub fn new() -> Self {
        Self(Vec::default(), false)
    }

    // Create a bitset from an iterator of chunks
    pub fn from_chunks_iter(iter: impl Iterator<Item = usize>) -> Self {
        Self(iter.collect(), false)
    }

    // Create a bitset from an iterator of booleans
    pub fn from_iter(iter: impl Iterator<Item = bool>) -> Self {
        let chunks = iter.chunks(usize::BITS as usize);
        let chunks = chunks
            .into_iter()
            .map(|chunk| chunk.fold(0, |accum, bit| accum << 1 | (bit as usize)));
        Self(chunks.collect(), false)
    }

    // Get an immutable reference to the stored chunks
    pub fn chunks(&self) -> &[usize] {
        self.0.as_slice()
    }
    
    // Get a mutable reference to the stored chunks
    pub fn chunks_mut(&mut self) -> &mut [usize] {
        self.0.as_mut_slice()
    }

    // Get the chunk and bitmask location for a specific chunk
    fn coords(index: usize) -> (usize, usize) {
        let chunk = index / (usize::BITS as usize);
        let location = index % (usize::BITS as usize);
        (chunk, location)
    }

    // Set a bit value in the bitset
    pub fn set(&mut self, index: usize) {
        let (chunk, location) = Self::coords(index);
        let chunk = &mut self.0[chunk];
        *chunk |= 1usize << location;
    }

    // Set the whole bitset to a single value
    pub fn splat(&mut self, value: bool) {
        for chunk in self.0.iter_mut() {
            *chunk = if value { usize::MAX } else { usize::MIN };
        }

        // We must store the value of the splat because we might allocate new chunks
        self.1 = value;
    }

    // Remove a bit value from the bitset
    pub fn remove(&mut self, index: usize) {
        let (chunk, location) = Self::coords(index);
        let chunk = &mut self.0[chunk];
        *chunk &= !(1usize << location);
    }

    // Get a bit value from the bitset
    pub fn get(&self, index: usize) -> bool {
        let (chunk, location) = Self::coords(index);
        (self.0[chunk] >> location) & 1 == 1
    }

    // Count the number of zeros in this bitset
    pub fn count_zeros(&self) -> usize {
        self.0
            .iter()
            .map(|chunk| chunk.count_zeros() as usize)
            .sum()
    }

    // Count the number of ones in this bitset
    pub fn count_ones(&self) -> usize {
        self.0.iter().map(|chunk| chunk.count_ones() as usize).sum()
    }
}
