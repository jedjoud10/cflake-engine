use crate::{registry, Archetype, ArchetypeSet, BorrowedItem, Component, ComponentStateSet, Mask, MaskSet, QueryError, QueryLayout};
use std::{collections::HashSet, ffi::c_void, marker::PhantomData, ptr::NonNull, rc::Rc};

// Per component type data
pub(crate) type PtrRow = Vec<Option<NonNull<c_void>>>;

// This cache contains multiple pointers to the component storages for faster iteration
pub struct QueryCache {
    // 64 unique rows that each contain a vector to store the component pointers
    pub(crate) rows: [PtrRow; 64],

    // Final two rows contain lengths and states
    pub(crate) lengths: Vec<usize>,
    pub(crate) states: Vec<ComponentStateSet>,
    archetypes: MaskSet,
}

impl Default for QueryCache {
    fn default() -> Self {
        const DEFAULT: PtrRow = Vec::new();
        Self {
            rows: [DEFAULT; 64], 

            archetypes: Default::default(),
        }
    }
}

impl QueryCache {
    // Update the cache using some archetypes
    pub(crate) fn update(&mut self, archetypes: &mut ArchetypeSet) {
        // Only certain archetypes are useful
        for (_, archetype) in archetypes.iter_mut() {
            // Insert the chunk if it is not present
            if !self.archetypes.contains(&archetype.mask) {
                self.rows.iter_mut().for_each(|row| row.push(None));
                self.lengths.push(0);
                self.archetypes.insert(archetype.mask);
                self.states.push(archetype.states.clone());

                // Chunk len, horizontal
                archetype.cache_index = self.lengths.len() - 1;
            }

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