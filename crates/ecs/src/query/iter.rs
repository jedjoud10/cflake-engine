use std::{marker::PhantomData, iter::FilterMap};

use crate::{Archetype, EcsManager, LayoutAccess, QueryLayout, ComponentStateRow, ArchetypeSet, Input};

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

// Return value of QueryIter
pub struct QueryItem<'a, Layout: QueryLayout<'a>> {
    // Values that will be read/written to
    tuple: Layout,

    // Current component states
    state: ComponentStateRow,

    // The archetype it came from
    archetype: &'a Archetype,
}

// Custom query iterator that will return QueryItem
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

}

impl<'a, Layout: QueryLayout<'a>> QueryIter<'a, Layout> {
    // Create a new iterator from some archetypes
    pub fn new(archetypes: &'a ArchetypeSet) -> Self {
        // Cache the layout mask for later use
        let access = Layout::combined();
        let (mask, _) = (access.reading(), access.writing());

        // Load the archetypes that validate our layout masks, and get their pointers as well
        let chunks = archetypes
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
        }
    }
}

impl<'a, Layout: QueryLayout<'a>> Iterator for QueryIter<'a, Layout> {
    type Item = QueryItem<'a, Layout>;

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
        let old = chunk
            .archetype
            .states
            .update(self.bundle, |row| row.update(|mutated, _| *mutated = *mutated | self.access.writing()))
            .unwrap();
        self.bundle += 1;

        // Create the query item
        let item = QueryItem {
            tuple: bundle,
            state: old,
            archetype: chunk.archetype,
        };

        Some(item)
    }
}

// Create a query without a filter
pub fn query<'a, Layout: QueryLayout<'a> + 'a>(archetypes: &'a ArchetypeSet) -> impl Iterator<Item = Layout> + 'a {
    QueryIter::new(archetypes).map(|item| item.tuple)
}

// Create a query with a filter
pub fn filtered<'a, Layout: QueryLayout<'a> + 'a>(archetypes: &'a ArchetypeSet, filter: fn(Input) -> bool) -> impl Iterator<Item = Layout> + 'a {
    QueryIter::new(archetypes).filter_map(move |item| if filter(Input(&item.state)) { Some(item.tuple) } else { None })
}
