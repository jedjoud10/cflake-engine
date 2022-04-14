use std::{marker::PhantomData, rc::Rc};

use crate::{ComponentStateSet, Mask, QueryCache, QueryError, QueryLayout, PtrReaderChunk};

// Custom query iterator
pub struct QueryIter<'a, Layout: QueryLayout<'a>> {
    // Iterator shit
    readers: Vec<PtrReaderChunk<'a, Layout>>,

    // Current main index, bundle index, and chunk index
    bundle: usize,
    chunk: usize,

    // Currently loaded values
    loaded: Option<PtrReaderChunk<'a, Layout>>,
}

impl<'a, Layout: QueryLayout<'a>> QueryIter<'a, Layout> {
    // Creates a new iterator using the cache
    pub fn new(cache: &'a QueryCache) -> Result<Self, QueryError> {
        let readers = PtrReaderChunk::<'a, Layout>::query(cache)?;
        let first = readers.get(0).cloned();
        Ok(Self {
            readers: readers,
            bundle: 0,
            chunk: 0,
            loaded: first,
        })
    }
}

impl<'a, Layout: QueryLayout<'a>> Iterator for QueryIter<'a, Layout> {
    type Item = Layout::SafeTuple;

    fn next(&mut self) -> Option<Self::Item> {
        // Handle empty cases
        if self.loaded.is_none() {
            return None;
        }

        // Try to load an element, and if we fail, move to the next chunk
        if let None = self.loaded.as_ref().unwrap().get(self.bundle) {
            // Reached the end of the chunk, move to the next one
            self.chunk += 1;
            let chunk = self.readers.get(self.chunk)?.clone();
            self.loaded.replace(chunk);
            self.bundle = 0;
        }

        // Actually load the element now (this must never fail)
        let element = self.loaded.as_ref().unwrap().get(self.bundle).unwrap();
        self.bundle += 1;
        Some(element)
    }
}
