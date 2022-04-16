use crate::{Mask, PtrReaderChunk, QueryCache, QueryError, QueryLayout};

// Custom query iterator
pub struct QueryIter<'a, Layout: QueryLayout<'a>> {
    // Readers from the query cache
    readers: Vec<PtrReaderChunk<'a, Layout>>,

    // Writing mask that will be overwritten in the componnent states
    writing_mask: Mask,

    // Indices
    bundle: usize,
    chunk: usize,

    // Currently loaded chunk reader
    loaded: Option<PtrReaderChunk<'a, Layout>>,
}

impl<'a, Layout: QueryLayout<'a>> QueryIter<'a, Layout> {
    // Creates a new iterator using the cache
    pub fn new(cache: &'a QueryCache) -> Result<Self, QueryError> {
        let (readers, writing_mask) = PtrReaderChunk::<'a, Layout>::query(cache)?;
        let first = readers.get(0).cloned();
        Ok(Self {
            readers,
            writing_mask,
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
        self.loaded.as_ref()?;

        // Try to load an element, and if we fail, move to the next chunk
        if self.loaded.as_ref().unwrap().get(self.bundle).is_none() {
            // Reached the end of the chunk, move to the next one
            self.chunk += 1;
            let chunk = self.readers.get(self.chunk)?.clone();
            self.loaded.replace(chunk);
            self.bundle = 0;
        }

        // Actually load the element by offsetting the base pointers
        let loaded = self.loaded.as_ref().unwrap();
        loaded.set_states(self.bundle, self.writing_mask);
        let element = loaded.get(self.bundle).unwrap();
        self.bundle += 1;
        Some(element)
    }
}
