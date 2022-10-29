use smallvec::SmallVec;

// Simple bitset that allocates using u64 chunks
// This bitset contains a specific number of elements per chunk
pub struct Bitset(SmallVec<[usize; 2]>, bool);

impl Bitset {
    // Create a new empty bit set
    pub fn new() -> Self {
        Self(SmallVec::default(), false)
    }

    // Get the chunk and bitmask location for a specific chunk
    fn coords(index: usize) -> (usize, usize) {
        let chunk = index / (usize::BITS as usize);
        let location = chunk % (usize::BITS as usize);
        (chunk, location)
    }

    // Set a bit value in the bitset
    pub fn set(&mut self, index: usize) {
        let (chunk, location) = Self::coords(index);
        let chunk = &mut self.0[chunk];
        *chunk |= 1usize << index; 
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
        *chunk &= !(1usize << index); 
    }

    // Get a bit value from the bitset
    pub fn get(&self, index: usize) -> bool {
        let (chunk, location) = Self::coords(index);
        (self.0[chunk] >> index) & 1 == 1
    }
}