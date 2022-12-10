use itertools::Itertools;
use parking_lot::MappedRwLockReadGuard;
use parking_lot::MappedRwLockWriteGuard;
use parking_lot::RwLock;
use parking_lot::RwLockReadGuard;
use parking_lot::RwLockWriteGuard;
use std::fmt::Debug;
use std::fmt::Display;
use std::sync::atomic::AtomicBool;
use std::sync::atomic::AtomicUsize;
use std::sync::atomic::Ordering;

// Simple atomic bitset that allocates using usize chunks
// This bitset contains a specific number of elements per chunk that we can share in multiple threads
#[derive(Default)]
pub struct AtomicBitSet(RwLock<Vec<AtomicUsize>>, AtomicBool);

impl AtomicBitSet {
    // Create a new empty bit set
    pub fn new() -> Self {
        Self(RwLock::new(Vec::default()), AtomicBool::new(false))
    }

    // Create a bitset from an iterator of chunks
    pub fn from_chunks_iter(
        iter: impl Iterator<Item = usize>,
    ) -> Self {
        Self(RwLock::new(iter.map(|s| AtomicUsize::new(s)).collect()), AtomicBool::new(false))
    }

    // Create a bitset from an iterator of booleans
    pub fn from_iter(iter: impl Iterator<Item = bool>) -> Self {
        let chunks = iter.chunks(usize::BITS as usize);
        let chunks = chunks.into_iter().map(|chunk| {
            chunk.fold(0, |accum, bit| accum << 1 | (bit as usize))
        });
        Self::from_chunks_iter(chunks)
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
    pub fn chunks(&self) -> MappedRwLockReadGuard<[AtomicUsize]> {
        RwLockReadGuard::map(self.0.read(), |s| s.as_slice())
    }

    // Get a mutable reference to the stored chunks
    pub fn chunks_mut(&self) -> MappedRwLockWriteGuard<[AtomicUsize]> {
        RwLockWriteGuard::map(self.0.write(), |s| s.as_mut_slice())
    }

    // Get the chunk and bitmask location for a specific chunk
    fn coords(index: usize) -> (usize, usize) {
        let chunk = index / (usize::BITS as usize);
        let location = index % (usize::BITS as usize);
        (chunk, location)
    }

    // Set a bit value in the bitset
    pub fn set(&self, index: usize, order: Ordering) {
        let (chunk, location) = Self::coords(index);

        // Extend the layer if needed (this bitset is dynamic)
        let len = self.0.read().len();
        if chunk >= len {
            let splat = if self.1.load(Ordering::Relaxed) { usize::MAX } else { usize::MIN };
            let num = chunk - len;
            self.0.write().extend((0..(num + 1)).into_iter().map(|_| AtomicUsize::new(splat)));
        }

        // Set the bit value specified in the chunk
        let chunk = &self.0.read()[chunk];
        chunk.fetch_or(1usize << location, order);
    }

    // Set the whole bitset to a single value
    pub fn splat(&self, value: bool, order: Ordering) {
        for chunk in &*self.chunks() {
            chunk.store(if value { usize::MAX } else { usize::MIN }, order);
        }

        // We must store the value of the splat because we might allocate new chunks
        self.1.store(value, Ordering::Relaxed);
    }

    // Remove a bit value from the bitset
    pub fn remove(&self, index: usize, order: Ordering) {
        let (chunk, location) = Self::coords(index);
        let chunk = &self.0.read()[chunk];
        chunk.fetch_and(!(1usize << location), order);
    }

    // Get a bit value from the bitset
    pub fn get(&self, index: usize, order: Ordering) -> bool {
        let (chunk, location) = Self::coords(index);

        self.0
            .read()
            .get(chunk)
            .map(|chunk| (chunk.load(order) >> location) & 1 == 1)
            .unwrap_or_default()
    }

    // Count the number of zeros in this bitset
    pub fn count_zeros(&self, order: Ordering) -> usize {
        self.0
            .read()
            .iter()
            .map(|chunk| chunk.load(Ordering::Relaxed).count_zeros() as usize)
            .sum()
    }

    // Count the number of ones in this bitset
    pub fn count_ones(&self, order: Ordering) -> usize {
        self.0.read().iter().map(|chunk| chunk.load(Ordering::Relaxed).count_ones() as usize).sum()
    }

    // Starting from a specific index, read forward and check if there is any set bits
    // Returns None if it could not find an set bit, returns Some with it's index if it did
    pub fn find_one_from(&self, index: usize, order: Ordering) -> Option<usize> {
        let (start_chunk, start_location) = Self::coords(index);
        self.chunks()
            .iter()
            .enumerate()
            .skip(start_chunk)
            .map(|(i, chunk)| (i, chunk.load(order)))
            .filter(|(_, chunk)| *chunk != 0)
            .filter_map(|(i, chunk)| {
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
    pub fn find_zero_from(&self, index: usize, order: Ordering) -> Option<usize> {
        let (start_chunk, start_location) = Self::coords(index);
        self.chunks()
            .iter()
            .enumerate()
            .skip(start_chunk)
            .map(|(i, chunk)| (i, chunk.load(order)))
            .filter(|(_, chunk)| *chunk != 0)
            .filter_map(|(i, chunk)| {
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

impl Display for AtomicBitSet {
    fn fmt(
        &self,
        f: &mut std::fmt::Formatter<'_>,
    ) -> std::fmt::Result {
        for chunk in &*self.chunks() {
            write!(f, "{:b}", chunk.load(Ordering::Relaxed))?;
        }

        std::fmt::Result::Ok(())
    }
}

impl Debug for AtomicBitSet {
    fn fmt(
        &self,
        f: &mut std::fmt::Formatter<'_>,
    ) -> std::fmt::Result {
        Display::fmt(self, f)
    }
}
