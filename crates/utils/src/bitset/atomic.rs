use atomic_traits::Atomic;
use atomic_traits::Bitwise;
use itertools::Itertools;
use num_traits::PrimInt;
use parking_lot::MappedRwLockReadGuard;

use parking_lot::RwLock;
use parking_lot::RwLockReadGuard;

use std::fmt::Binary;
use std::fmt::Debug;
use std::fmt::Display;
use std::mem::size_of;
use std::sync::atomic::Ordering;

// Simple atomic bitset that allocates using usize chunks
// This bitset contains a specific number of elements per chunk that we can share in multiple threads
#[derive(Default)]
pub struct AtomicBitSet<T: Bitwise>(RwLock<Vec<T>>)
where
    <T as Atomic>::Type: PrimInt;

// Gets the value that represents 0 for the specific integer
fn zero<T: Atomic>() -> <T as Atomic>::Type
where
    T::Type: PrimInt,
{
    <<T as Atomic>::Type as num_traits::identities::Zero>::zero()
}

// Gets the value that represents 1 for the specific integer
fn one<T: Atomic>() -> <T as Atomic>::Type
where
    T::Type: PrimInt,
{
    <<T as Atomic>::Type as num_traits::identities::One>::one()
}

// Gets the value that represents the smallest number possible for the specific interger
fn min<T: Atomic>() -> <T as Atomic>::Type
where
    T::Type: PrimInt,
{
    <<T as Atomic>::Type as num_traits::Bounded>::min_value()
}

impl<T: Bitwise> AtomicBitSet<T>
where
    <T as Atomic>::Type: PrimInt,
{
    // Create a new empty bit set
    pub fn new() -> Self {
        Self(RwLock::new(Vec::default()))
    }

    // Create a bit set with some pre-allocated chunks
    pub fn with_capacity(elements: usize) -> Self {
        let chunk = (elements as f32 / Self::bitsize() as f32).ceil() as usize;
        Self(RwLock::new(Vec::with_capacity(chunk)))
    }

    // Create a bitset from an iterator of chunks
    pub fn from_chunks_iter(iter: impl Iterator<Item = <T as Atomic>::Type>) -> Self {
        Self(RwLock::new(iter.map(T::new).collect()))
    }

    // Get the bit-size of the primitive
    pub fn bitsize() -> usize {
        size_of::<<T as Atomic>::Type>() * 8
    }

    // Create a bitset from an iterator of booleans
    pub fn from_iter(iter: impl Iterator<Item = bool>) -> Self {
        let chunks = iter.chunks(Self::bitsize());

        let chunks = chunks.into_iter().map(|chunk| {
            chunk.fold(zero::<T>(), |accum, bit| {
                accum << 1 | (if bit { one::<T>() } else { zero::<T>() })
            })
        });
        Self::from_chunks_iter(chunks)
    }

    // Get an immutable reference to the stored chunks
    pub fn chunks(&self) -> MappedRwLockReadGuard<[T]> {
        RwLockReadGuard::map(self.0.read(), |s| s.as_slice())
    }

    // Get the chunk and bitmask location for a specific chunk
    fn coords(index: usize) -> (usize, usize) {
        let chunk = index / (Self::bitsize());
        let location = index % (Self::bitsize());
        (chunk, location)
    }

    // Extend the inner chunks with a specific count
    fn extend(&self, count: usize) {
        if count > 0 {
            let splat = min::<T>();
            self.0.write().extend((0..(count)).map(|_| T::new(splat)));
        }
    }

    // Set a bit value in the bitset
    pub fn set(&self, index: usize, order: Ordering) {
        let (chunk, location) = Self::coords(index);

        // Extend the layer if needed (this bitset is dynamic)
        let len = self.0.read().len();
        if chunk >= len {
            self.extend((chunk - len) + 1);
        }

        // Set the bit value specified in the chunk
        let chunk = &self.0.read()[chunk];
        chunk.fetch_or(one::<T>() << location, order);
    }

    /*
    // Set a range within the bitset to a specific value
    pub fn splat(&self, range: std::ops::Range<usize>, value: bool, order: Ordering) {
        let (start, end) = (range.start, range.end);
        let (start_chunk, start_location) = Self::coords(start);
        let (end_chunk, end_location) = Self::coords(end);

        fn splatting<T: Bitwise>(atomic: &T, start: usize, end: usize, value: bool, order: Ordering) where <T as Atomic>::Type: PrimInt {
            if value {
                let inv = crate::enable_in_range::<<T as Atomic>::Type>(start, end);
                atomic.fetch_or(inv, order);
            } else {
                let inv = todo!();
                atomic.fetch_and(inv, order);
            }
        }

        // Extend to make sure we have enough
        let len = self.0.read().len();
        if end_chunk >= len {
            self.extend((end_chunk - len) + 1);
        }

        // If we start partially within a chunk, set it
        if start_location != 0 {
            let atomic = &self.0.read()[start_chunk];
            splatting(atomic, start_location, Self::bitsize(), value, order);
        }

        // If we end partially within a chunk, set it
        if end_location != 0 {
            let atomic = &self.0.read()[end_chunk];
            splatting(atomic, 0, end_location, value, order);
        }

        // Set the region within it
    }
    */

    // Remove a bit value from the bitset
    pub fn remove(&self, index: usize, order: Ordering) {
        let (chunk, location) = Self::coords(index);
        if let Some(chunk) = &self.0.read().get(chunk) {
            chunk.fetch_and(!(one::<T>() << location), order);
        }
    }

    // Pre-allocate a specific amount of elements
    pub fn reserve(&mut self, elements: usize) {
        let additional = (elements as f32 / Self::bitsize() as f32).ceil() as usize;
        self.extend(additional);
    }

    // Get a bit value from the bitset
    pub fn get(&self, index: usize, order: Ordering) -> bool {
        let (chunk, location) = Self::coords(index);

        self.0
            .read()
            .get(chunk)
            .map(|chunk| (chunk.load(order) >> location) & one::<T>() == one::<T>())
            .unwrap_or_default()
    }

    // Count the number of zeros in this bitset
    pub fn count_zeros(&self, order: Ordering) -> usize {
        self.0
            .read()
            .iter()
            .map(|chunk| chunk.load(order).count_zeros() as usize)
            .sum()
    }

    // Count the number of ones in this bitset
    pub fn count_ones(&self, order: Ordering) -> usize {
        self.0
            .read()
            .iter()
            .map(|chunk| chunk.load(order).count_ones() as usize)
            .sum()
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
            .filter(|(_, chunk)| *chunk != zero::<T>())
            .filter_map(|(i, chunk)| {
                let offset = i * Self::bitsize();
                let result = if i == start_chunk {
                    // Starting chunk, take start_location in consideration
                    let inverted = !((one::<T>() << start_location) - one::<T>());
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
    pub fn find_zero_from(&self, index: usize, order: Ordering) -> Option<usize> {
        let (start_chunk, start_location) = Self::coords(index);
        self.chunks()
            .iter()
            .enumerate()
            .skip(start_chunk)
            .map(|(i, chunk)| (i, chunk.load(order)))
            .filter(|(_, chunk)| *chunk != zero::<T>())
            .filter_map(|(i, chunk)| {
                let offset = i * Self::bitsize();
                let result = if i == start_chunk {
                    // Starting chunk, take start_location in consideration
                    let inverted = (one::<T>() << start_location) - one::<T>();
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

impl<T: Bitwise> Display for AtomicBitSet<T>
where
    <T as Atomic>::Type: Debug + PrimInt + Binary,
{
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        for chunk in &*self.chunks() {
            write!(f, "{:b}", chunk.load(Ordering::Relaxed))?;
        }

        std::fmt::Result::Ok(())
    }
}

impl<T: Bitwise> Debug for AtomicBitSet<T>
where
    <T as Atomic>::Type: Debug + PrimInt + Binary,
{
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        Display::fmt(self, f)
    }
}
