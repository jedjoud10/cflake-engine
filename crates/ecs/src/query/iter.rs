use std::{marker::PhantomData, rc::Rc};

use crate::{AccessMask, ComponentStateSet, Mask, QueryCache, QueryError, QueryLayout, CacheChunk};

// Custom query iterator
pub struct QueryIter<'a, Layout: QueryLayout<'a>> {
    // Iterator shit
    access: AccessMask,
    chunks: Vec<CacheChunk<'a, Layout>>,
    _phantom: PhantomData<&'a Layout>,

    // Current main index, bundle index, and chunk index
    bundle: usize,
    chunk: usize,

    // Currently loaded values
    loaded: Option<CacheChunk<'a, Layout>>,
}

impl<'a, Layout: QueryLayout<'a>> QueryIter<'a, Layout> {
    // Creates a new iterator using the cache
    pub fn new(cache: &'a QueryCache) -> Result<Self, QueryError> {
        Ok(Self {
            access: Layout::layout_access_mask().map_err(QueryError::ComponentError)?,
            chunks: todo!(),
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
        if self.chunks.is_empty() {
            return None;
        }
        self.loaded.get_or_insert_with(|| {
            todo!()
        });

        // We've reached the end of the current chunk, reset
        if self.bundle == self.loaded.as_ref().unwrap().length {
            self.bundle = 0;
            self.chunk += 1;
            self.loaded = None;

            // Check if we've reached the end of the query
            if self.chunk == self.chunks.len() {
                return None;
            }
        }

        // Update the component mutation states
        let chunk = self.loaded.as_ref().unwrap();
        chunk.states.set(self.bundle, self.access.writing);

        // Read the pointers
        self.bundle += 1;
        Some(Layout::read_tuple(chunk.ptrs, self.bundle))
    }
}
