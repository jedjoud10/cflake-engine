use std::marker::PhantomData;

use crate::{Archetype, ArchetypeSet, Evaluate, ItemInput, LayoutAccess, QueryLayout, StateRow, Mask, States};

// This is what is returned from the iterator
pub struct QueryItem<'a, L: QueryLayout<'a>> {
    tuple: L,
    state: StateRow,
    archetype_mask: Mask,
    _phantom: PhantomData<&'a L>,
}

// Chunks contain basic information about the current archetype
struct Chunk<'a, L: QueryLayout<'a>> {
    archetype_mask: Mask,
    ptrs: L::PtrTuple,
    states: States,
    len: usize,
}

impl<'a, L: QueryLayout<'a>> Clone for Chunk<'a, L> {
    fn clone(&self) -> Self {
        Self { archetype_mask: self.archetype_mask.clone(), ptrs: self.ptrs.clone(), states: self.states.clone(), len: self.len.clone() }
    }
}

// Custom query iterator that will return QueryItem
pub struct QueryIter<'a, L: QueryLayout<'a>> {
    chunks: Vec<Chunk<'a, L>>,
    access: LayoutAccess,
    bundle: usize,
    chunk: usize,
    loaded: Option<Chunk<'a, L>>,
}

impl<'a, L: QueryLayout<'a>> QueryIter<'a, L> {
    // Create a new iterator from some archetypes
    pub fn new(archetypes: &'a mut ArchetypeSet) -> Self {
        // Cache the layout mask for later use
        let access = L::combined();
        let (mask, _) = (access.reading(), access.writing());

        // Load the archetypes that validate our layout masks, and get their pointers as well
        let chunks = archetypes
            .iter_mut()
            .filter_map(|(_, archetype)| {
                (archetype.mask() & mask == mask).then(|| {
                    Chunk {
                        archetype_mask: archetype.mask(),
                        len: archetype.len(),
                        states: archetype.states().clone(),
                        ptrs: L::try_fetch_ptrs(archetype).unwrap(),
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

impl<'a, L: QueryLayout<'a>> Iterator for QueryIter<'a, L> {
    type Item = QueryItem<'a, L>;

    fn next(&mut self) -> Option<Self::Item> {
        // Move to the next chunk if possible, and return the new/old chunk reference
        let chunk = if self.bundle == self.loaded.as_ref()?.len {
            self.chunk += 1;
            let chunk = *self.chunks.get(self.chunk)?;
            self.loaded.replace(chunk);
            self.bundle = 0;
            self.loaded.as_ref()
        } else {
            self.loaded.as_ref()
        }?;

        // Dereference the layout pointers
        let bundle = unsafe { L::read_as_layout_at(chunk.ptrs, self.bundle) };
        
        // Update the bundle states
        let old = chunk
            .states
            .update(self.bundle, |mutated, _| {
                *mutated = *mutated | self.access.writing()
            })
            .unwrap();
        self.bundle += 1;

        // Create the query item and return it
        Some(QueryItem {
            tuple: bundle,
            state: old,
            archetype_mask: chunk.archetype_mask,
            _phantom: Default::default(),
        })
    }
}

pub(super) fn query<'a, L: QueryLayout<'a> + 'a>(
    archetypes: &'a mut ArchetypeSet,
) -> impl Iterator<Item = L> + 'a {
    QueryIter::new(archetypes).map(|item| item.tuple)
}

pub(super) fn filtered<'a, L: QueryLayout<'a> + 'a, Filter: Evaluate>(
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
