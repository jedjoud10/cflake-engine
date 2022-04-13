use std::{marker::PhantomData, rc::Rc};

use crate::{QueryCache, QueryError, QueryLayout, ComponentStateSet, Mask};

// Custom query iterator
pub struct QueryIter<'a, Layout: QueryLayout<'a>> {
    mask: Mask,
    tuples: Vec<Layout::PtrTuple>,
    lengths: &'a [usize],
    states: &'a [Rc<ComponentStateSet>],

    // Current main index, bundle index, and chunk index
    bundle: usize,
    chunk: usize,

    // Currently loaded values
    loaded: Option<(Layout::PtrTuple, usize)>,
}

impl<'a, Layout: QueryLayout<'a>> QueryIter<'a, Layout> {
    // Creates a new iterator using the cache
    pub fn new(cache: &'a QueryCache) -> Result<Self, QueryError> {
        Ok(Self {
            mask: Layout::layout_mask().map_err(QueryError::ComponentError)?,
            tuples: Layout::get_filtered_chunks(cache)?,
            lengths: cache.lengths.as_slice(),
            states: cache.states.as_slice(),
            bundle: 0,
            chunk: 0,
            loaded: None,
        })
    }
}

impl<'a, Layout: QueryLayout<'a>> Iterator for QueryIter<'a, Layout> {
    type Item = Layout::SafeTuple;

    fn next(&mut self) -> Option<Self::Item> {
        // Try to load a new chunk
        if self.tuples.is_empty() {
            return None;
        }
        self.loaded.get_or_insert_with(|| (self.tuples[self.chunk], self.lengths[self.chunk]));

        // We've reached the end of the current chunk, reset
        if self.bundle == self.loaded.unwrap().1 {
            self.bundle = 0;
            self.chunk += 1;
            self.loaded = None;

            // Check if we've reached the end of the query
            if self.chunk == self.tuples.len() {
                return None;
            }
        }

        // Read the pointers
        self.bundle += 1;
        Some(Layout::read_tuple(self.loaded.unwrap().0, self.bundle))
    }
}
