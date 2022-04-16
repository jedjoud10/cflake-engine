use crate::{Mask, PtrReaderChunk, QueryCache, QueryLayout, LayoutAccess};

// Custom query iterator
pub struct QueryIter<'a, Layout: QueryLayout<'a>> {
    // Readers from the query cache
    readers: Vec<PtrReaderChunk<'a, Layout>>,

    // How we shall access the components
    access: LayoutAccess,

    // Indices
    bundle: usize,
    chunk: usize,

    // Currently loaded chunk reader
    loaded: Option<PtrReaderChunk<'a, Layout>>,
}

impl<'a, Layout: QueryLayout<'a>> QueryIter<'a, Layout> {
    // Creates a new query iterator using the cache
    pub fn new(cache: &'a QueryCache) -> Self {
        // Cache the layout mask for later use
        let access = Layout::combined();
        let (&mask, &writing) = (access.reading(), access.writing());

        // Get all the cache chunks
        let chunks = cache.view();

        // Create the readers
        let readers = chunks
            .iter()
            .filter_map(|chunk| {
                // Check if the chunk's mask validates the layout's mask
                (chunk.mask & mask == mask).then(|| PtrReaderChunk::new(chunk))
            })
            .collect::<Vec<_>>();

        let first = readers.get(0).cloned();
        Self {
            readers,
            access,
            bundle: 0,
            chunk: 0,
            loaded: first,
        }
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
        loaded.update_states(self.bundle, self.access);
        let element = loaded.get(self.bundle).unwrap();
        self.bundle += 1;
        Some(element)
    }
}
