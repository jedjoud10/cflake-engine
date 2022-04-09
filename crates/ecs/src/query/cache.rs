use parking_lot::RwLock;
use smallvec::SmallVec;
use tinyvec::ArrayVec;

use crate::{Mask, MaskHasher, QueryLayout, Archetype, StorageVecPtr, registry, Component};
use std::{
    any::Any,
    collections::{hash_map::Entry, HashMap}, sync::atomic::AtomicPtr, ffi::c_void,
};


#[derive(Default)]
pub(crate) struct QueryCache {
    // Waste of memory but it works decently
    rows: ArrayVec<[Vec<Option<*mut c_void>>; 64]>,
    lengths: Vec<usize>,
}

impl QueryCache {
    // Register a new archetype into the cache
    pub fn insert(&mut self, archetype: &mut Archetype) {
        // Insert the pointers
        for (offset, row) in self.rows.iter_mut().enumerate() {
            let mask = Mask::from_offset(offset);            
            
            // Get the component vector's pointer
            let ptr = archetype.vectors[&mask].get_ptr();
            row.push(Some(ptr));
        }
        // And the length
        self.lengths.push(archetype.entities.len());
    }

    // Update the cache using some new archetypes
    pub fn update(&mut self, archetypes: &[Archetype]) {
        // Update the pointers and chunk lengths
        for archetype in archetypes {
            // Get the corresponding chunk for each component type
            for (offset, row) in self.rows.iter_mut().enumerate() {
                let mask = Mask::from_offset(offset);
                
                // Overwrite el pointer
                let ptr = &mut row[archetype.query_cache_index];
                *ptr = Some(archetype.vectors[&mask].get_ptr())
            }
        }
    }

    // Get the row for a specific component type
    pub fn get_row<T: Component>(&self) -> &Vec<Option<*mut c_void>> {
        &self.rows[registry::mask::<T>().unwrap().offset()]
    }
}
