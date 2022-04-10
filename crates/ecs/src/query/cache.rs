use parking_lot::RwLock;
use smallvec::SmallVec;
use tinyvec::ArrayVec;

use crate::{registry, Archetype, Component, Mask, MaskHasher, QueryLayout, StorageVecPtr};
use std::{
    any::Any,
    collections::{hash_map::Entry, HashMap, HashSet},
    ffi::c_void,
    sync::atomic::AtomicPtr,
};

#[derive(Default)]
pub struct QueryCache {
    // Waste of memory but it works decently
    rows: ArrayVec<[Vec<Option<*mut c_void>>; 64]>,
    lengths: Vec<usize>,
    archetypes: HashSet<Mask, MaskHasher>,
}

impl QueryCache {
    // Inserts or updates an archetype, depending if it is currently present in the cache
    pub fn update(&mut self, archetype: &mut Archetype) {
        // Insert the chunk if it is not present
        if !self.archetypes.contains(&archetype.mask) {
            self.rows.iter_mut().for_each(|row| row.push(None));
            self.lengths.push(0);
        }

        // Always update the chunk length and rows
        let idx = archetype.query_cache_index;
        self.lengths[archetype.query_cache_index] = archetype.entities.len();

        for (offset, row) in self.rows.iter_mut().enumerate() {
            let mask = Mask::from_offset(offset);

            // Get the component vector's pointer only if it valid in the archetype
            if let Some(vector) = archetype.vectors.get(&mask) {
                row[idx].replace(vector.get_ptr());
            }
        }
    } 

    // Get the row for a specific component type
    pub fn get_row<T: Component>(&self) -> &[Option<*mut c_void>] {
        let offset = registry::mask::<T>().unwrap().offset();
        let row = &self.rows[offset];
        row
    }

    // Get the lengths for each chunk
    pub fn get_lengths(&self) -> &[usize] {
        &self.lengths
    }
}
