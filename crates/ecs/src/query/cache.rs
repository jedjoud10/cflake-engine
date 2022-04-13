use crate::{registry, Archetype, ArchetypeSet, BorrowedItem, Component, ComponentStateSet, Mask, MaskSet, QueryError, QueryLayout};
use std::{collections::HashSet, ffi::c_void, marker::PhantomData, ptr::NonNull, rc::Rc};

type StoragePtr = Option<NonNull<c_void>>;

// A query cache chunk (column) that contains the raw pointers, length, and states
pub struct QueryChunk {
    mask: Mask,
    ptrs: [StoragePtr; 64],
    len: usize,
    states: Rc<ComponentStateSet>,
}

impl QueryChunk {
    // From an archetype
    pub fn new(archetype: &mut Archetype) -> Self {
        const DEFAULT: StoragePtr = StoragePtr::None;
        Self {
            // It's fine if they are empty, since we will initialize them while updating
            ptrs: [DEFAULT; 64],
            len: 0,
            
            mask: archetype.mask,
            states: archetype.states.clone(),
        }
    }
}

// This cache contains multiple pointers to the component storages for faster iteration
#[derive(Default)]
pub struct QueryCache {
    // AoS for simplicty here
    chunks: Vec<QueryChunk>,
}

impl QueryCache {
    // Update the cache using some archetypes
    pub(crate) fn update(&mut self, archetypes: &mut ArchetypeSet) {
        // Only certain archetypes are useful
        for (_, archetype) in archetypes.iter_mut() {
            // Insert the chunk if it is not present
            let idx = archetype.cache_index.get_or_insert_with(|| {
                self.chunks.push(QueryChunk::new(archetype));
                self.chunks.len() - 1
            });

            // Always update the archetype chunk
            let chunk = &mut self.chunks[*idx];
            chunk.len = archetype.entities.len();
            // Update the component storage pointers
            for (offset, old) in chunk.ptrs.iter_mut().enumerate().take(registry::count()) {
                let mask = Mask::from_offset(offset);

                // Update the pointer
                *old = archetype.vectors.get(&mask).map(|(_, x)| *x);
            }
        }
    }

    // Get all the archetype chunks
    pub(super) fn view(&self) -> &[QueryChunk] {
        self.chunks.as_slice()
    }
}