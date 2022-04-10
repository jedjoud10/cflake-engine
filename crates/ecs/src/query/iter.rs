use std::marker::PhantomData;

use crate::{QueryLayout, QueryCache};

// Custom query iterator
pub struct QueryIter<'a, Layout: QueryLayout<'a>> {
    tuples: Vec<(Layout::PtrTuple, usize)>,
    _phantom: PhantomData<&'a ()>,

    // Maximum number of bundles
    count: usize,

    // Current main index, bundle index, and chunk index
    bundle: usize,
    chunk: usize,

    // Currently loaded values
    loaded: Option<(Layout::PtrTuple, usize)>,
}

impl<'a, Layout: QueryLayout<'a>> QueryIter<'a, Layout> {
    // Creates a new iterator using the cache
    pub fn new(cache: &'a QueryCache) -> Self {
        Self {
            tuples: Layout::get_filtered_chunks(cache),
            _phantom: Default::default(),
            count: 0,
            bundle: 0,
            chunk: 0,
            loaded: None,
        }
    }
}

impl<'a, Layout: QueryLayout<'a>> Iterator for QueryIter<'a, Layout> {
    type Item = Layout::SafeTuple;

    fn next(&mut self) -> Option<Self::Item> {
        // We've reached the end of the query
        if self.chunk == self.tuples.len() {
            return None;
        }

        // Try to load a new chunk
        self.loaded.get_or_insert_with(|| self.tuples[self.chunk]);

        // We've reached the end of the current chunk, reset
        if self.bundle == self.loaded.unwrap().1 {
            self.bundle = 0;
            self.chunk += 1;
            self.loaded = None;
        }

        // Read the pointers
        self.bundle += 1;
        Some(Layout::read(self.loaded.unwrap().0))
    }

    fn size_hint(&self) -> (usize, Option<usize>) {
        (self.count, Some(self.count))
    }
}

impl<'a, Layout: QueryLayout<'a>> ExactSizeIterator for QueryIter<'a, Layout> {}
