use crate::{registry, Archetype, ArchetypeSet, BorrowedItem, Component, ComponentStateSet, Mask, MaskSet, QueryError, QueryLayout};
use std::{collections::HashSet, ffi::c_void, marker::PhantomData, ptr::NonNull, rc::Rc};

type StoragePtr = Option<NonNull<c_void>>;

// A query cache chunk (column) that contains the raw pointers, length, and states
pub struct QueryCacheChunk {
    ptrs: [StoragePtr; 64],
    len: usize,
    states: Rc<ComponentStateSet>,
}

impl QueryCacheChunk {
    // From an archetype
    pub fn new(archetype: &mut Archetype) -> Self {
        Self {
            // It's fine if they are empty, since we will initialize them while updating
            ptrs: Default::default(),
            len: 0,
            states: archetype.states.clone(),
        }
    }
}

// This cache contains multiple pointers to the component storages for faster iteration
#[derive(Default)]
pub struct QueryCache {
    // AoS for simplicty here
    chunks: Vec<QueryCacheChunk>,
}

impl QueryCache {
    // Update the cache using some archetypes
    pub(crate) fn update(&mut self, archetypes: &mut ArchetypeSet) {
        // Only certain archetypes are useful
        for (_, archetype) in archetypes.iter_mut() {
            // Insert the chunk if it is not present
            let idx = archetype.cache_index.get_or_insert_with(|| {
                self.chunks.push(QueryCacheChunk::new(archetype));
                self.chunks.len() - 1
            });

            // Always update the archetype chunk
            let chunk = &mut self.chunks[*idx];
            todo!()
            /*
            // Always update the chunk length and rows
            let idx = archetype.cache_index;
            self.lengths[archetype.cache_index] = archetype.entities.len();

            // Update the component rows
            for (offset, row) in self.rows.iter_mut().enumerate().take(registry::count()) {
                let mask = Mask::from_offset(offset);

                // Get the component vector's pointer only if it valid in the archetype
                if let Some((_, ptr)) = archetype.vectors.get(&mask) {
                    row[idx].replace(*ptr);
                }
            }
            */
        }
    }

    // Get a view into the cache for a specific component
    pub(super) fn view<'b, 'a, T: BorrowedItem<'a>>(&'b self) -> Result<&'b [Option<NonNull<c_void>>], QueryError> {
        let offset = registry::mask::<T::Component>().map_err(QueryError::ComponentError)?.offset();
        let ptrs = &self.rows[offset];
        dbg!(self.lengths.as_slice());
        Ok(ptrs.as_slice())
    }
}

// Query chunk to be used inside the layouts
pub(crate) struct QueryChunk<'a, Layout: QueryLayout<'a>> {
    pub(crate) length: usize,
    pub(crate) states: Rc<ComponentStateSet>,
    pub(crate) ptrs: Layout::PtrTuple
}