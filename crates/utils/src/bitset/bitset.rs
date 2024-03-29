use itertools::Itertools;
use num_traits::PrimInt;
use std::fmt::Binary;
use std::fmt::Debug;
use std::fmt::Display;
use std::mem::size_of;

// Simple bitset that allocates using usize chunks
// This bitset contains a specific number of elements per chunk
#[derive(Default, Clone)]
pub struct BitSet<T: PrimInt>(Vec<T>);

impl<T: PrimInt> BitSet<T> {
    // Create a new empty bit set
    pub fn new() -> Self {
        Self(Vec::default())
    }

    // Create a bit set with some pre-allocated chunks
    pub fn with_capacity(elements: usize) -> Self {
        let chunks = (elements as f32 / Self::bitsize() as f32).ceil() as usize;
        Self(Vec::with_capacity(chunks))
    }

    // Create a bitset from an iterator of chunks
    pub fn from_chunks_iter(iter: impl Iterator<Item = T>) -> Self {
        Self(iter.collect())
    }

    // Get the bit-size of the primitive
    pub fn bitsize() -> usize {
        size_of::<T>() * 8
    }

    // Create a bitset from an iterator of booleans
    pub fn from_iter(iter: impl Iterator<Item = bool>) -> Self {
        let chunks = iter.chunks(Self::bitsize());
        let chunks = chunks.into_iter().map(|chunk| {
            chunk.fold(T::zero(), |accum, bit| {
                accum << 1 | (if bit { T::one() } else { T::zero() })
            })
        });
        Self::from_chunks_iter(chunks)
    }

    // Get an immutable reference to the stored chunks
    pub fn chunks(&self) -> &[T] {
        self.0.as_slice()
    }

    // Get a mutable reference to the stored chunks
    pub fn chunks_mut(&mut self) -> &mut [T] {
        self.0.as_mut_slice()
    }

    // Get the chunk and bitmask location for a specific chunk
    fn coords(index: usize) -> (usize, usize) {
        let chunk = index / (Self::bitsize());
        let location = index % (Self::bitsize());
        (chunk, location)
    }

    // Extend the inner chunks with a specific count
    fn extend(&mut self, count: usize) {
        if count > 0 {
            let splat = T::min_value();
            self.0.extend((0..(count)).map(|_| splat));
        }
    }

    // Set a bit value in the bitset
    pub fn set(&mut self, index: usize) {
        let (chunk, location) = Self::coords(index);

        // Extend the layer if needed (this bitset is dynamic)
        if chunk >= self.0.len() {
            self.extend((chunk - self.0.len()) + 1);
        }

        // Set the bit value specified in the chunk
        let chunk = &mut self.0[chunk];
        *chunk = *chunk | (T::one() << location);
    }

    // Remove a bit value from the bitset
    pub fn remove(&mut self, index: usize) {
        let (chunk, location) = Self::coords(index);
        let chunk = &mut self.0[chunk];
        *chunk = *chunk & !(T::one() << location);
    }

    // Pre-allocate a specific amount of elements
    pub fn reserve(&mut self, elements: usize) {
        let additional = (elements as f32 / Self::bitsize() as f32).ceil() as usize;
        self.extend(additional);
    }

    // Get a bit value from the bitset
    pub fn get(&self, index: usize) -> bool {
        let (chunk, location) = Self::coords(index);

        self.0
            .get(chunk)
            .map(|chunk| (*chunk >> location) & T::one() == T::one())
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
            .filter(|(_, chunk)| **chunk != T::zero())
            .filter_map(|(i, &chunk)| {
                let offset = i * Self::bitsize();
                let result = if i == start_chunk {
                    // Starting chunk, take start_location in consideration
                    let inverted = !((T::one() << start_location) - T::one());
                    (chunk & inverted).trailing_zeros() as usize + offset
                } else {
                    // Dont care, start at 0 as index
                    chunk.trailing_zeros() as usize + offset
                };

                (result != (offset + Self::bitsize())).then_some(result)
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
            .filter(|(_, chunk)| **chunk != T::zero())
            .filter_map(|(i, &chunk)| {
                let offset = i * Self::bitsize();
                let result = if i == start_chunk {
                    // Starting chunk, take start_location in consideration
                    let inverted = (T::one() << start_location) - T::one();
                    (chunk | inverted).trailing_ones() as usize + offset
                } else {
                    // Dont care, start at 0 as index
                    chunk.trailing_ones() as usize + offset
                };

                (result != (offset + Self::bitsize())).then_some(result)
            })
            .next()
    }
}

impl<T: PrimInt + Binary> Display for BitSet<T> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        for chunk in self.0.iter() {
            write!(f, "{:b}", *chunk)?;
        }

        std::fmt::Result::Ok(())
    }
}

impl<T: PrimInt + Binary> Debug for BitSet<T> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        Display::fmt(self, f)
    }
}
