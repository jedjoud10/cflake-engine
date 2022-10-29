use std::ops::{Range, RangeInclusive};

// Number of layers when usize is a 32 bit integer
#[cfg(target_pointer_width = "32")]
const LAYER_COUNT: usize = 5;

// Number of layers when usize is a 64 bit integer
#[cfg(target_pointer_width = "64")]
const LAYER_COUNT: usize = 9;

// Default number of chunks that are allocated by default
const DEFAULT_BASE_CHUNK_CAPACITY: usize = 1;

// Hierchical bitset. Heavily inspired from the hibitset crate
// This hiearchichal bitset is mostly used in the world crate and ECS crate
// This HiBitSet is stored on the heap since there is quite a bit of stuff to store
// layer 0: max 1 element
// layer 1: max 64 or 32 elements
// layer 2: max 4096 or 1024 element
// layer n: max 64^n or 32^n elements
pub struct HiBitSet([Vec<usize>; LAYER_COUNT], bool);

impl HiBitSet {
    // Create a new empty hierchichal bit set
    pub fn new() -> Self {
        let vector = (0..LAYER_COUNT)
            .into_iter()
            .map(|i| {
                let len = (DEFAULT_BASE_CHUNK_CAPACITY as f32 * usize::BITS as f32) / (usize::BITS.saturating_pow(i as u32) as f32);
                let len = len.ceil() as usize;
                vec![0usize; len]
            })
            .collect::<Vec<Vec<usize>>>();
        Self(vector.try_into().unwrap(), false)
    }

    // Get a specific layer using it's index
    pub fn layer(&self, layer: usize) -> &Vec<usize> {
        &self.0[layer]
    }

    // Get a specific layer mutably using it's index
    pub fn layer_mut(&mut self, layer: usize) -> &mut Vec<usize> {
        &mut self.0[layer]
    }

    // Get the chunk and bitmask location for a specific layer
    fn coords(index: usize, layer: usize) -> (usize, usize) {
        let threshold = usize::BITS.saturating_pow(layer as u32 + 1) as usize;
        let chunk = index / threshold;

        let location = if layer == 0 { index % threshold } else {
            index / usize::BITS.saturating_pow(index as u32) as usize
        };
        (chunk, location)
    }

    // Set a bit value in the hibitset
    pub fn set(&mut self, index: usize) {
        let splat = if self.1 { usize::MAX } else { usize::MIN };
        for i in 0..LAYER_COUNT {
            let layer = self.layer_mut(i);
            let (chunk, location) = Self::coords(index, i);           

            // Extend the layer if needed (this bitset is dynamic)
            if chunk >= (layer.len() * usize::BITS as usize) {                
                let num = chunk - layer.len();
                layer.extend(std::iter::repeat(splat).take(num));
            }

            // Add the bit to the chunk
            let bits = &mut layer[chunk];
            *bits |= 1 << location;
        }
    }

    // Set the whole bitset to a single value
    pub fn splat(&mut self, value: bool) {
        for i in 0..LAYER_COUNT {
            let layer = self.layer_mut(i);
            for bits in layer {
                *bits = if value { usize::MAX } else { usize::MIN }; 
            }
        }

        // We must store the value of the splat because we might allocate new chunks
        self.1 = value;
    }

    // Remove a bit value from the hibitset
    pub fn remove(&mut self, index: usize) {
        for i in (0..(LAYER_COUNT-1)).rev() {
            if i == 0 {
                let layer = self.layer_mut(i);
                let (chunk, location) = Self::coords(index, i);

                // No need to do anything if the index doesn't even exist
                if location >= layer.len() {
                    return;
                }

                // Update the bitmask
                let bits = &mut layer[chunk];
                *bits &= !(1 << location);
            } else {
                // Get the parent layer index and location
                let threshold = usize::BITS.saturating_pow((i+2) as u32) as usize;
                let parent_chunk = index / threshold;
                let parent_location = index % threshold;

                // Calculate the start and end indices for the current layer
                let current_start = parent_chunk * (usize::BITS as usize);
                let current_end = parent_chunk * (usize::BITS as usize + 1);

                // Update the parent of the current layer
                let current_layer = self.layer(i);
                let set = current_layer
                    [current_start..current_end]
                    .iter()
                    .any(|bitmask| *bitmask != usize::MIN);
                
                // Update the parent layer
                let parent_layer = self.layer_mut(i+1);
                let parent_bits = &mut parent_layer[parent_chunk];

                // Update the bitmask
                if set {
                    *parent_bits |= 1 << parent_location;
                } else {
                    *parent_bits &= !(1 << parent_location);
                }
            }
        }
    }

    // Get a bit value from the hibitset
    pub fn get(&self, index: usize) -> bool {
        !(0..LAYER_COUNT).any(|i| {
            let layer = self.layer(i);
            let (chunk, location) = Self::coords(index, i);
            layer.get(chunk).map(|bits| {
                ((*bits >> location) & 1) == 0
            }).unwrap_or_default()
        }) 
    }
}

impl Default for HiBitSet {
    fn default() -> Self {
        Self::new()
    }
}
