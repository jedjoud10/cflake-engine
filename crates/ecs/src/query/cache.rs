use parking_lot::RwLock;
use smallvec::SmallVec;

use crate::{Mask, MaskHasher, QueryLayout, Archetype, StorageVecPtr};
use std::{
    any::Any,
    collections::{hash_map::Entry, HashMap}, sync::atomic::AtomicPtr, ffi::c_void,
};

// Small vec for storing pointers to the component vec columns
type ColumnPtrs = SmallVec<[(Mask, Option<StorageVecPtr>); 8]>;
type Chunk = (usize, ColumnPtrs);

// Query cache that stores a local copy of the generated queries, so we can iterate through them more efficiently
pub struct QueryCache {
    // Layout mask -> Chunks
    collumns: HashMap<Mask, Vec<Chunk>, MaskHasher>,
}

impl QueryCache {
    // Creates a new chunk from an arhcetype and a layout entry mask
    fn new_chunk(layout: Mask, archetype: &Archetype) -> Chunk {
        let ptrs = archetype
            .vectors
            .iter()
            .filter_map(|(&mask, column)| 
                // It must validate the layout mask
                if mask & layout == Default::default() { None }
                else { Some((mask, column.get_ptr())) }
            )
        .collect::<ColumnPtrs>();

        return (archetype.entities.len(), ptrs)
    }
    // Makes a new collumn that can contains multiple chunks
    fn insert_default(&mut self, mask: Mask) {
        self.collumns.entry(mask).or_default();
    }
    // Update the internal cache using the archetype we initialized it with
    pub fn update<'a, Layout: QueryLayout<'a>>(&mut self, archetype: &mut Archetype) -> Option<()> {
        // Validate the collumn if it is new
        let mask = Layout::layout_mask().ok()?;
        self.insert_default(mask);

        /*
        // Fetch the specific chunk
        let collumn = self.collumns.get_mut(&mask)?;
        let idx = archetype.query_cache_indices[&mask];
        let (len, chunk) = collumn.get_mut(idx)?;

        // Update the chunk data
        *len = archetype.entities.len();
        for (mask, column) in archetype.vectors.iter() {
            chunk[] 
        }
        */

        Some(())
    }
}
