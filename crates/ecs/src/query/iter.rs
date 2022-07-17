use std::marker::PhantomData;

use crate::{
    ArchetypeSet, Evaluate, ItemInput, LayoutAccess, Mask, QueryLayout, StateRow, States,
    ViewLayout,
};

// Raw data that is returned from the query (mutable)
struct QueryItem<'a, L: QueryLayout<'a>> {
    tuple: L,
    state: StateRow,
    archetype_mask: Mask,
    _phantom: PhantomData<&'a L>,
}

// Raw data that is returned from the view query (immutable)
struct ViewItem<'a, L: ViewLayout<'a>> {
    tuple: L,
    state: StateRow,
    archetype_mask: Mask,
    _phantom: PhantomData<&'a L>,
}

// Chunk used for mutable query
struct Chunk<'a, L: QueryLayout<'a>> {
    ptrs: L::PtrTuple,
    states: States,
    len: usize,
}

// Chunk used for immutable query
struct ViewChunk<'a, L: ViewLayout<'a>> {
    ptrs: L::PtrTuple,
    states: States,
    len: usize,
}

// Custom mutable archetype iterator.
struct QueryIter<'a, L: QueryLayout<'a>> {
    chunks: Vec<Chunk<'a, L>>,
    access: LayoutAccess,
    bundle: usize,
    loaded: Option<Chunk<'a, L>>,
    len: usize,
}

impl<'a, L: QueryLayout<'a>> QueryIter<'a, L> {
    // Create a new mutable query iterator that will iterate through the valid archetypes entities
    // TODO: Make this less ugly
    fn new(archetypes: &'a mut ArchetypeSet) -> Self {
        let access = L::combined();
        let mask = access.shared() | access.unique();

        // Create a new vector containing the archetypes in arbitrary order
        let mut chunks = archetypes
            .iter_mut()
            .filter(|(m, _)| m.contains(mask))
            .map(|(_, archetype)| Chunk {
                len: archetype.len(),
                states: archetype.states().clone(),
                ptrs: L::try_fetch_ptrs(archetype).unwrap(),
            })
            .collect::<Vec<_>>();

        // Get the maximum number of bundles that we have
        let len = chunks.iter().map(|chunk| chunk.len).sum();

        // Create and initiate the iterator
        let last = chunks.pop();
        Self {
            chunks,
            access,
            bundle: 0,
            loaded: last,
            len,
        }
    }
}

// Custom immutable archetype iterator.
struct ViewIter<'a, L: ViewLayout<'a>> {
    chunks: Vec<ViewChunk<'a, L>>,
    mask: Mask,
    bundle: usize,
    loaded: Option<ViewChunk<'a, L>>,
    len: usize,
}

impl<'a, L: ViewLayout<'a>> ViewIter<'a, L> {
    // Create a new immutable query iterator that will iterate through the valid archetypes entities
    // TODO: Make this less ugly
    fn new(archetypes: &'a ArchetypeSet) -> Self {
        let mask = L::combined();

        // Create a new vector containing the archetypes in arbitrary order
        let mut chunks = archetypes
            .iter()
            .filter(|(m, _)| m.contains(mask))
            .map(|(_, archetype)| ViewChunk {
                len: archetype.len(),
                states: archetype.states().clone(),
                ptrs: unsafe { L::try_fetch_ptrs(archetype).unwrap() },
            })
            .collect::<Vec<_>>();

        // Get the maximum number of bundles that we have
        let len = chunks.iter().map(|chunk| chunk.len).sum();

        // Create and initiate the iterator
        let last = chunks.pop();
        Self {
            chunks,
            mask,
            bundle: 0,
            loaded: last,
            len,
        }
    }
}

// Implement the iterator trait for mutable queries
impl<'a, L: QueryLayout<'a>> Iterator for QueryIter<'a, L> {
    type Item = QueryItem<'a, L>;

    fn next(&mut self) -> Option<Self::Item> {
        // Move to the next chunk if possible
        if self.bundle == self.loaded.as_ref()?.len {
            self.loaded = self.chunks.pop();
            self.bundle = 0;
        }

        // Dereference the pointer
        let chunk = self.loaded.as_ref()?;
        let bundle = unsafe { L::read_as_layout_at(chunk.ptrs, self.bundle) };

        // Update the bundle state
        let old = chunk
            .states
            .update(self.bundle, |mutated, _| {
                *mutated = *mutated | self.access.unique()
            })
            .unwrap();
        self.bundle += 1;

        // Create the query item and return it
        Some(QueryItem {
            tuple: bundle,
            state: old,
            archetype_mask: self.access.shared() | self.access.unique(),
            _phantom: Default::default(),
        })
    }

    fn size_hint(&self) -> (usize, Option<usize>) {
        (self.len, Some(self.len))
    }
}

impl<'a, L: QueryLayout<'a>> ExactSizeIterator for QueryIter<'a, L> {}

// Implement the iterator trait for immutable view queries
impl<'a, L: ViewLayout<'a>> Iterator for ViewIter<'a, L> {
    type Item = ViewItem<'a, L>;

    fn next(&mut self) -> Option<Self::Item> {
        // Move to the next chunk if possible
        if self.bundle == self.loaded.as_ref()?.len {
            self.loaded = self.chunks.pop();
            self.bundle = 0;
        }

        // Dereference the pointer
        let chunk = self.loaded.as_ref()?;
        let bundle = unsafe { L::read_as_layout_at(chunk.ptrs, self.bundle) };

        // Get the bundle state
        let state = chunk.states.get(self.bundle).unwrap();
        self.bundle += 1;

        // Create the query item and return it
        Some(ViewItem {
            tuple: bundle,
            state,
            archetype_mask: self.mask,
            _phantom: Default::default(),
        })
    }

    fn size_hint(&self) -> (usize, Option<usize>) {
        (self.len, Some(self.len))
    }
}

impl<'a, L: ViewLayout<'a>> ExactSizeIterator for ViewIter<'a, L> {}

// Fetch a query, assuming that the layout is valid
pub unsafe fn query_unchecked<'a, L: QueryLayout<'a> + 'a>(
    archetypes: &'a mut ArchetypeSet,
) -> impl ExactSizeIterator<Item = L> + 'a {
    QueryIter::new(archetypes).map(|item| item.tuple)
}

// Fetch a view query, assuming that the layout is valid (it is always valid)
pub unsafe fn view<'a, L: ViewLayout<'a> + 'a>(
    archetypes: &'a ArchetypeSet,
) -> impl ExactSizeIterator<Item = L> + 'a {
    ViewIter::new(archetypes).map(|item| item.tuple)
}

// Fetch a filtered query, assuming the the layout is valid
pub unsafe fn query_filtered<'a, L: QueryLayout<'a> + 'a, Filter: Evaluate>(
    archetypes: &'a mut ArchetypeSet,
    _: Filter,
) -> impl Iterator<Item = L> + 'a {
    let cache = Filter::setup();

    QueryIter::new(archetypes).filter_map(move |item| {
        Filter::eval(
            &cache,
            &ItemInput {
                state_row: item.state,
                mask: item.archetype_mask,
            },
        )
        .then_some(item.tuple)
    })
}

// Fetch a filtered view query, assuming the layout is valid
pub unsafe fn view_filtered<'a, L: ViewLayout<'a> + 'a, Filter: Evaluate>(
    archetypes: &'a ArchetypeSet,
    _: Filter,
) -> impl Iterator<Item = L> + 'a {
    let cache = Filter::setup();

    ViewIter::new(archetypes).filter_map(move |item| {
        Filter::eval(
            &cache,
            &ItemInput {
                state_row: item.state,
                mask: item.archetype_mask,
            },
        )
        .then_some(item.tuple)
    })
}
