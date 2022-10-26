use std::ops::{RangeInclusive, Range};

// Hierchical bitset. Heavily inspired from the hibitset crate
// This hiearchichal bitset is mostly used in the world crate and ECS crate
// This HiBitSet is stored on the heap since there is quite a bit of stuff to store
// MAX INDEX: 4,294,967,295
// MAX INDEX 2: 18,446,744,073,709,551,615
pub struct HiBitSet([Vec<u16>; 8]);

// Only used for counting
pub enum Direction {
    Left, Right
}

impl HiBitSet {
    // Create a new empty hierchichal bit set
    pub fn new() -> Self {
        let vector = (0..8).into_iter().map(|i| vec![0u16; 16usize.pow(i)]).collect::<Vec<Vec<u16>>>();
        Self(vector.try_into().unwrap())
    }

    // Get a specific layer using it's index
    pub fn layer(&self, layer: usize) -> &[u16] {
        &self.0[layer]
    }

    // Get a specific layer mutably using it's index
    pub fn layer_mut(&mut self, layer: usize) -> &mut [u16] {
        &mut self.0[layer]
    }

    // Set a bit value in the hibitset
    pub fn set(&mut self, bit: bool, index: u32) {
        todo!()
    }

    // Get a bit value from the hibitset
    pub fn get(&self, index: u32) -> bool {
        todo!()
    }
    
    // Count the number of zeros till we reach a valid one starting from an index
    pub fn count_zeros_until_one(&self, index: u32, direction: Direction) -> u32 {
        todo!()
    }
    
    // Count the number of ones till we reach a valid zero starting from an index
    pub fn count_ones_until_zero(&self, index: u32, direction: Direction) -> u32 {
        todo!()
    }

    // Count the number of set bits in a specific range
    pub fn count_ones_in_range(&self, range: Range<u32>) -> u32 {
        todo!()
    }

    // Count the number of unset bits in a specific range
    pub fn count_zeros_in_range(&self, range: Range<u32>) -> u32 {
        todo!()
    }
}

impl Default for HiBitSet {
    fn default() -> Self {
        Self::new()
    }
}