use itertools::Itertools;
use std::fmt::Debug;
use std::fmt::Display;

// Simple bitset that allocates using u64 chunks
// This bitset contains a specific number of elements per chunk
#[derive(Default, Clone)]
pub struct BitSet(Vec<usize>, bool);

impl BitSet {
    // Create a new empty bit set
    pub fn new() -> Self {
        Self(Vec::default(), false)
    }

    // Create a bitset from an iterator of chunks
    pub fn from_chunks_iter(
        iter: impl Iterator<Item = usize>,
    ) -> Self {
        Self(iter.collect(), false)
    }

    // Create a bitset from an iterator of booleans
    pub fn from_iter(iter: impl Iterator<Item = bool>) -> Self {
        let chunks = iter.chunks(usize::BITS as usize);
        let chunks = chunks.into_iter().map(|chunk| {
            chunk.fold(0, |accum, bit| accum << 1 | (bit as usize))
        });
        Self(chunks.collect(), false)
    }

    // Create a bitset using a specific function and the number of elements
    pub fn from_pattern(
        callback: impl FnMut(usize) -> bool,
        count: usize,
    ) -> Self {
        let iter = (0..count).into_iter().map(callback);
        Self::from_iter(iter)
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

        // Extend the layer if needed (this bitset is dynamic)
        if chunk >= self.0.len() {
            let splat = if self.1 { usize::MAX } else { usize::MIN };
            let num = chunk - self.0.len();
            self.0.extend(std::iter::repeat(splat).take(num + 1));
        }

        // Set the bit value specified in the chunk
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

        self.0
            .get(chunk)
            .map(|chunk| (chunk >> location) & 1 == 1)
            .unwrap_or_default()
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

    // Starting from a specific index, read forward and check if there is any set bits
    // Returns None if it could not find an set bit, returns Some with it's index if it did
    pub fn find_one_from(&self, index: usize) -> Option<usize> {
        let (start_chunk, start_location) = Self::coords(index);
        self.chunks()
            .iter()
            .enumerate()
            .skip(start_chunk)
            .filter(|(_, chunk)| **chunk != 0)
            .filter_map(|(i, &chunk)| {
                let offset = i * usize::BITS as usize;
                let result = if i == start_chunk {
                    // Starting chunk, take start_location in consideration
                    let inverted = !((1 << start_location) - 1);
                    (chunk & inverted).trailing_zeros() as usize
                        + offset
                } else {
                    // Dont care, start at 0 as index
                    chunk.trailing_zeros() as usize + offset
                };

                (result != (offset + 64)).then_some(result)
            })
            .next()
    }

    // Starting from a specific index, read forward and check if there is any unset bits
    // Returns None if it could not find an unset bit, returns Some with it's index if it did
    pub fn find_zero_from(&self, index: usize) -> Option<usize> {
        let (start_chunk, start_location) = Self::coords(index);
        self.chunks()
            .iter()
            .enumerate()
            .skip(start_chunk)
            .filter(|(_, chunk)| **chunk != 0)
            .filter_map(|(i, &chunk)| {
                let offset = i * usize::BITS as usize;
                let result = if i == start_chunk {
                    // Starting chunk, take start_location in consideration
                    let inverted = (1 << start_location) - 1;
                    (chunk | inverted).trailing_ones() as usize
                        + offset
                } else {
                    // Dont care, start at 0 as index
                    chunk.trailing_ones() as usize + offset
                };

                (result != (offset + 64)).then_some(result)
            })
            .next()
    }
}

impl Display for BitSet {
    fn fmt(
        &self,
        f: &mut std::fmt::Formatter<'_>,
    ) -> std::fmt::Result {
        for chunk in self.0.iter() {
            write!(f, "{:b}", *chunk)?;
        }

        std::fmt::Result::Ok(())
    }
}

impl Debug for BitSet {
    fn fmt(
        &self,
        f: &mut std::fmt::Formatter<'_>,
    ) -> std::fmt::Result {
        Display::fmt(self, f)
    }
}
