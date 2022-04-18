use std::marker::PhantomData;

use crate::{Archetype, EcsManager, LayoutAccess, QueryLayout};

// Currently loaded chunk
struct Chunk<'a, Layout: QueryLayout<'a>> {
    // Loaded archetype
    archetype: &'a Archetype,

    // And respective layout ptrs
    ptrs: Layout::PtrTuple,
}

impl<'a, Layout: QueryLayout<'a>> Clone for Chunk<'a, Layout> {
    fn clone(&self) -> Self {
        Self {
            archetype: self.archetype.clone(),
            ptrs: self.ptrs,
        }
    }
}

impl<'a, Layout: QueryLayout<'a>> Copy for Chunk<'a, Layout> {}

impl<'a, Layout: QueryLayout<'a>> Chunk<'a, Layout> {
    // Load a component bundle from a chunk and also set it's respective mutation states
    fn load(&self, bundle: usize) -> Layout {
        Layout::offset(self.ptrs, bundle)
    }

    // Check if a bundle index is valid
    fn check_bundle(&self, bundle: usize) -> bool {
        bundle < self.archetype.len()
    }
}

// Custom query iterator
pub struct QueryIter<'a, Layout: QueryLayout<'a>> {
    // Chunks that contains the archetypes and base ptrs
    chunks: Vec<Chunk<'a, Layout>>,

    // How we shall access the components
    access: LayoutAccess,

    // Indices
    bundle: usize,
    chunk: usize,

    // Currently loaded archetype and base ptrs
    loaded: Option<Chunk<'a, Layout>>,

    _b: PhantomData<Layout>,
}

impl<'a, Layout: QueryLayout<'a>> QueryIter<'a, Layout> {
    // Create a new iterator from the main manager
    pub(crate) fn new(manager: &'a EcsManager) -> Self {
        // Cache the layout mask for later use
        let access = Layout::combined();
        let (mask, _) = (access.reading(), access.writing());

        // Load the archetypes that validate our layout masks, and get their pointers as well
        let chunks = manager
            .archetypes
            .iter()
            .filter_map(|(_, archetype)| {
                (archetype.mask & mask == mask).then(|| {
                    // Combine the archetype and pointers into a chunk
                    Chunk {
                        archetype,
                        ptrs: Layout::get_base_ptrs(archetype),
                    }
                })
            })
            .collect::<Vec<_>>();

        // Load the first chunk that is valid (order is irrelevant)
        let first = chunks.first().cloned();

        Self {
            chunks,
            access,
            bundle: 0,
            chunk: 0,
            loaded: first,
            _b: PhantomData::default(),
        }
    }
}

impl<'a, Layout: QueryLayout<'a>> Iterator for QueryIter<'a, Layout> {
    type Item = Layout;

    fn next(&mut self) -> Option<Self::Item> {
        // Handle empty cases
        self.loaded.as_ref()?;

        // Check if the bundle index is valid, and move to the next archetype if it isn't
        if !self.loaded.as_ref().unwrap().check_bundle(self.bundle) {
            // Reached the end of the archetype chunk, move to the next one
            self.chunk += 1;
            let chunk = *self.chunks.get(self.chunk)?;
            self.loaded.replace(chunk);
            self.bundle = 0;
        }

        // Load a component bundle
        let chunk = self.loaded.as_ref().unwrap();
        let bundle = chunk.load(self.bundle);
        // Update the bundle states
        chunk
            .archetype
            .states
            .update(self.bundle, |row| row.update(|mutated, _| *mutated = *mutated | self.access.writing()))
            .unwrap();
        self.bundle += 1;

        // TODO: Handle filters
        //todo!();

        Some(bundle)
    }
}
