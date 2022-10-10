use crate::{
    ArchetypeSet, Entity, Evaluate, LayoutAccess, Mask, MutQueryLayout, RefQueryLayout, StateRow, Archetype,
};
use std::{cell::RefCell, marker::PhantomData, rc::Rc};

// Raw data that is returned from the query (immutable)
pub struct RefQueryItemResult<'a, L: RefQueryLayout<'a>> {
    tuple: L,
    state: StateRow,
    archetype_mask: Mask,
    entity: Entity,
    _phantom: PhantomData<&'a ()>,
}

impl<'a, L: RefQueryLayout<'a>> RefQueryItemResult<'a, L> {
    // Get the data tuple immutably
    pub fn data(&self) -> &L {
        &self.tuple
    }

    // Get the state row
    pub fn state_row(&self) -> StateRow {
        self.state
    }

    // Get the archetype mask
    pub fn archetype_mask(&self) -> Mask {
        self.archetype_mask
    }

    // Get the entity ID
    pub fn entity(&self) -> Entity {
        self.entity
    }
}

// Chunk used for immutable query
pub struct RefQueryChunk<'c, 'a, L: RefQueryLayout<'a>> {
    ptrs: L::PtrTuple,
    access: LayoutAccess,
    states: Rc<RefCell<Vec<StateRow>>>,
    entities: &'c [Entity],
    len: usize,
}

// Custom immutable archetype iterator.
pub(crate) struct RefQueryIter<'c, 'a, L: RefQueryLayout<'a>> {
    chunks: Vec<RefQueryChunk<'c, 'a, L>>,
    index: usize,
    loaded: Option<RefQueryChunk<'c, 'a, L>>,
    len: usize,
}

// Implement the iterator trait for immutable queries
impl<'c, 'a, L: RefQueryLayout<'a>> Iterator for RefQueryIter<'c, 'a, L> {
    type Item = RefQueryItemResult<'a, L>;

    fn next(&mut self) -> Option<Self::Item> {
        // Move to the next chunk if possible
        if self.index == self.loaded.as_ref()?.len {
            self.loaded = self.chunks.pop();
            self.index = 0;
        }

        // Dereference the pointer
        let chunk = self.loaded.as_ref()?;
        let bundle = unsafe { L::read(chunk.ptrs, self.index) };

        // Get the bundle state
        let state = *chunk.states.borrow().get(self.index).unwrap();
        self.index += 1;

        // Create the query item and return it
        Some(RefQueryItemResult {
            tuple: bundle,
            state,
            archetype_mask: chunk.access.shared(),
            entity: chunk.entities[self.index - 1],
            _phantom: Default::default(),
        })
    }

    fn size_hint(&self) -> (usize, Option<usize>) {
        (self.len, Some(self.len))
    }
}

// Raw data that is returned from the query (mutable)
pub struct MutQueryItemResult<'s: 'l, 'l, L: MutQueryLayout<'s, 'l>> {
    tuple: L,
    state: StateRow,
    archetype_mask: Mask,
    entity: Entity,
    _phantom2: PhantomData<&'s ()>,
    _phantom3: PhantomData<&'l ()>,
}

impl<'s: 'l, 'l, L: MutQueryLayout<'s, 'l>> MutQueryItemResult<'s, 'l, L> {
    // Get the data tuple immutably
    pub fn data(&self) -> &L {
        &self.tuple
    }

    // Get the data tuple mutably
    pub fn data_mut(&mut self) -> &mut L {
        &mut self.tuple
    }

    // Get the state row
    pub fn state_row(&self) -> StateRow {
        self.state
    }

    // Get the archetype mask
    pub fn archetype_mask(&self) -> Mask {
        self.archetype_mask
    }

    // Get the entity ID
    pub fn entity(&self) -> Entity {
        self.entity
    }
}

// Chunk used for mutable query
struct MutQueryChunk<'s: 'l, 'l, L: MutQueryLayout<'s, 'l>> {
    layout: L::SliceTuple,
    access: LayoutAccess,
    states: Rc<RefCell<Vec<StateRow>>>,
    len: usize,
}

// Custom immutable archetype iterator.
pub(crate) struct MutQueryIter<'s: 'l, 'l, L: MutQueryLayout<'s, 'l>> {
    chunks: Vec<MutQueryChunk<'s, 'l, L>>,
    index: usize,
    len: usize,
}

// Implement the iterator trait for immutable view queries
impl<'s: 'l, 'l, L: MutQueryLayout<'s, 'l>> Iterator for MutQueryIter<'s, 'l, L> {
    type Item = MutQueryItemResult<'s, 'l, L>;

    fn next(&mut self) -> Option<Self::Item> {
        /*
        // Move to the next chunk if possible
        if self.index == self.loaded.as_ref()?.len {
            self.loaded = self.chunks.pop();
            self.index = 0;
        }

        // Dereference the pointer
        let chunk = self.loaded.as_ref()?;
        let bundle = unsafe { L::read(chunk.ptrs, self.index) };

        // Get the bundle state and update them
        let mut states = chunk.states.borrow_mut();
        let row = states.get_mut(self.index).unwrap();
        let state = row.update(|_, _, mutated| {
            *mutated = *mutated | chunk.access.unique();
        });
        self.index += 1;

        // Create the query item and return it
        Some(MutQueryItemResult {
            tuple: bundle,
            state,
            archetype_mask: chunk.access.shared() | chunk.access.unique(),
            entity: chunk.entities[self.index - 1],
            _phantom: PhantomData,
            _phantom2: PhantomData,
            _phantom3: PhantomData,
        })
        */
        None
    }

    fn size_hint(&self) -> (usize, Option<usize>) {
        (self.len, Some(self.len))
    }
}

// Immutable query that returns RefQueryItemResult
// TODO: Fix code duplication
/*
pub(crate) fn query_ref_raw<'c: 'a, 'a, L: RefQueryLayout<'a>>(
    archetypes: &'c ArchetypeSet,
) -> Option<RefQueryIter<'c, 'a, L>> {
    if !L::is_valid() {
        return None;
    }

    let mut chunks = archetypes
        .iter()
        .filter_map(|(m, archetype)| {
            L::access(*m)
                .and_then(|access| (*m != Mask::zero()).then_some(access))
                .map(|a| (a, archetype))
        })
        .map(|(access, archetype)| RefQueryChunk {
            len: archetype.len(),
            states: archetype.states(),
            ptrs: L::prepare(archetype).unwrap(),
            entities: archetype.entities(),
            access,
        })
        .collect::<Vec<_>>();

    let len = chunks.iter().map(|chunk| chunk.len).sum();
    let last = chunks.pop();
    Some(RefQueryIter {
        chunks,
        loaded: last,
        len,
        index: 0,
    })
}
*/

// Mutable query that returns MutQueryItemResult
pub(crate) fn query_mut_raw<'s: 'l, 'l, L: MutQueryLayout<'s, 'l>>(
    archetypes: &mut ArchetypeSet,
) -> Option<MutQueryIter<'s, 'l, L>> {
    if !L::is_valid() {
        return None;
    }

    let mut chunks = archetypes
        .iter_mut()
        .filter_map(|(m, archetype)| {
            L::access(*m)
                .and_then(|access| (*m != Mask::zero()).then_some(access))
                .map(|a| (a, archetype))
        })
        .map(|(access, archetype)| MutQueryChunk {
            len: archetype.len(),
            states: archetype.states(),
            layout: L::prepare(archetype).unwrap(),
            access,
        })
        .collect::<Vec<_>>();

    let len = chunks.iter().map(|chunk| chunk.len).sum();
    Some(MutQueryIter {
        chunks,
        len,
        index: 0,
    })
}

/*
// Immutable query that returns the layout tuple
pub(crate) fn query_ref_marked<'ars: 'ar, 'ar: 's, 's: 'l, 'l, L: RefQueryLayout<'ar, 's,>>(
    archetypes: &'c ArchetypeSet,
) -> Option<impl Iterator<Item = (L, Entity)> + 'a> {
    query_ref_raw::<L>(archetypes).map(|iter| iter.map(|item| (item.tuple, item.entity)))
}
*/

/*
// Immutable query that returns the layout tuple, filtered
pub(crate) fn query_ref_filter_marked<'c: 'a, 'a, L: RefQueryLayout<'a>, E: Evaluate>(
    archetypes: &'c ArchetypeSet,
    _filter: E,
) -> Option<impl Iterator<Item = (L, Entity)> + 'a> {
    let cached = E::prepare();
    query_ref_raw::<L>(archetypes).map(|iter| {
        iter.filter(move |item| E::eval(&cached, item.state, item.archetype_mask))
            .map(|item| (item.tuple, item.entity))
    })
}
*/

// Mutable query that returns the layout tuple, filtered
pub(crate) fn query_mut_marked<'s: 'l, 'l, L: MutQueryLayout<'s, 'l>>(
    archetypes: &mut ArchetypeSet,
) -> impl Iterator<Item = L> + 'l {
    query_mut_raw::<L>(archetypes).unwrap().map(|item| (item.tuple))
}
