use crate::{registry, Archetype, ArchetypeSet, BorrowedItem, Component, ComponentStateSet, Mask, MaskSet, QueryError};
use std::{collections::HashSet, ffi::c_void, rc::Rc};

pub struct QueryCache {
    // Waste of memory but it works decently
    rows: [Vec<Option<*mut c_void>>; 64],
    states: Vec<Rc<ComponentStateSet>>,
    lengths: Vec<usize>,
    archetypes: MaskSet,
}

impl Default for QueryCache {
    fn default() -> Self {
        const DEFAULT: Vec<Option<*mut c_void>> = Vec::new();
        Self {
            rows: [DEFAULT; 64],
            states: Default::default(),
            lengths: Default::default(),
            archetypes: Default::default(),
        }
    }
}

impl QueryCache {
    // Update the cache using some archetypes
    pub(crate) fn update(&mut self, archetypes: &mut ArchetypeSet) {
        // Only certain archetypes are useful
        for (_, archetype) in archetypes.iter_mut().filter(|(_, a)| a.cache_pending_update) {
            // Reset state
            archetype.cache_pending_update = false;

            // Insert the chunk if it is not present
            if !self.archetypes.contains(&archetype.mask) {
                self.rows.iter_mut().for_each(|row| row.push(None));
                self.lengths.push(0);
                self.archetypes.insert(archetype.mask);

                // Chunk len, horizontal
                archetype.cache_index = self.lengths.len() - 1;
            }

            // Always update the chunk length and rows
            let idx = archetype.cache_index;
            self.lengths[archetype.cache_index] = archetype.entities.len();

            for (offset, row) in self.rows.iter_mut().enumerate().take(registry::count()) {
                let mask = Mask::from_offset(offset);

                // Get the component vector's pointer only if it valid in the archetype
                if let Some((_, ptr)) = archetype.vectors.get(&mask) {
                    row[idx].replace(*ptr);
                }
            }
        }
    }

    // Get the row for a specific component type using a borrowed item
    pub(crate) fn get_row<'b, 'a, T: BorrowedItem<'a>>(&'b self) -> Result<&'b [Option<*mut c_void>], QueryError> {
        let offset = registry::mask::<T::Component>().map_err(QueryError::ComponentError)?.offset();
        let row = &self.rows[offset];

        // Write le mutation state here

        Ok(row)
    }

    // Get the lengths for each chunk
    pub(crate) fn get_lengths(&self) -> &[usize] {
        &self.lengths
    }
}
