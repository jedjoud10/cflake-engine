use crate::{registry, Archetype, Component, Mask, MaskHasher, QueryError};
use std::{collections::HashSet, ffi::c_void};

pub struct QueryCache {
    // Waste of memory but it works decently
    rows: [Vec<Option<*mut c_void>>; 64],
    lengths: Vec<usize>,
    archetypes: HashSet<Mask, MaskHasher>,
}

impl Default for QueryCache {
    fn default() -> Self {
        const DEFAULT: Vec<Option<*mut c_void>> = Vec::new();
        Self {
            rows: [DEFAULT; 64],
            lengths: Default::default(),
            archetypes: Default::default(),
        }
    }
}

impl QueryCache {
    // Inserts or updates an archetype, depending if it is currently present in the cache
    pub(crate) fn update(&mut self, archetype: &mut Archetype) {
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

        for (offset, row) in self.rows.iter_mut().enumerate() {
            let mask = Mask::from_offset(offset);

            // Get the component vector's pointer only if it valid in the archetype
            if let Some((_, ptr)) = archetype.vectors.get(&mask) {
                row[idx].replace(*ptr);
            }
        }
    }

    // Get the row for a specific component type
    pub(crate) fn get_row<T: Component>(&self) -> Result<&[Option<*mut c_void>], QueryError> {
        let offset = registry::mask::<T>().map_err(QueryError::ComponentError)?.offset();
        let row = &self.rows[offset];
        Ok(row)
    }

    // Get the lengths for each chunk
    pub(crate) fn get_lengths(&self) -> &[usize] {
        &self.lengths
    }
}
