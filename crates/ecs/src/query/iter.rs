use std::{marker::PhantomData, cell::RefCell, rc::Rc};
use crate::{StateRow, RefQueryLayout, Mask, EcsManager, MutQueryLayout, ArchetypeSet, LayoutAccess, Evaluate};

// Raw data that is returned from the query (immutable)
struct RefQueryItemResult<'a, L: RefQueryLayout<'a>> {
    tuple: L,
    state: StateRow,
    archetype_mask: Mask,
    _phantom: PhantomData<&'a ()>,
}

// Chunk used for immutable query
struct RefQueryChunk<'a, L: RefQueryLayout<'a>> {
    ptrs: L::PtrTuple,
    mask: Mask,
    states: Rc<RefCell<Vec<StateRow>>>,
    len: usize,
}

// Custom immutable archetype iterator.
struct RefQueryIter<'a, L: RefQueryLayout<'a>> {
    chunks: Vec<RefQueryChunk<'a, L>>,
    index: usize,
    loaded: Option<RefQueryChunk<'a, L>>,
    len: usize,
}

// Implement the iterator trait for immutable queries
impl<'a, L: RefQueryLayout<'a>> Iterator for RefQueryIter<'a, L> {
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
            archetype_mask: chunk.mask,
            _phantom: Default::default(),
        })
    }

    fn size_hint(&self) -> (usize, Option<usize>) {
        (self.len, Some(self.len))
    }
}

// Raw data that is returned from the query (mutable)
struct MutQueryItemResult<'a, L: MutQueryLayout<'a>> {
    tuple: L,
    state: StateRow,
    archetype_mask: Mask,
    _phantom: PhantomData<&'a ()>,
}

// Chunk used for mutable query
struct MutQueryChunk<'a, L: MutQueryLayout<'a>> {
    ptrs: L::PtrTuple,
    mask: Mask,
    states: Rc<RefCell<Vec<StateRow>>>,
    len: usize,
}

// Custom immutable archetype iterator.
struct MutQueryIter<'a, L: MutQueryLayout<'a>> {
    chunks: Vec<MutQueryChunk<'a, L>>,
    access: LayoutAccess,
    index: usize,
    loaded: Option<MutQueryChunk<'a, L>>,
    len: usize,
}

// Implement the iterator trait for immutable view queries
impl<'a, L: MutQueryLayout<'a>> Iterator for MutQueryIter<'a, L> {
    type Item = MutQueryItemResult<'a, L>;

    fn next(&mut self) -> Option<Self::Item> {
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
        let state = row.update(|added, mutated, removed| {
            *mutated = *mutated | self.access.unique();
        });
        self.index += 1;

        // Create the query item and return it
        Some(MutQueryItemResult {
            tuple: bundle,
            state,
            archetype_mask: chunk.mask,
            _phantom: Default::default(),
        })
    }

    fn size_hint(&self) -> (usize, Option<usize>) {
        (self.len, Some(self.len))
    }
}

// Immutable query that returns RefQueryItemResult
fn query_ref_raw<'a, L: RefQueryLayout<'a>>(archetypes: &ArchetypeSet) -> RefQueryIter<'a, L> {
    let access = L::access();
    let mask = access.shared() | access.unique();

    let mut chunks = archetypes
        .iter()
        .filter(|(m, _)| m.contains(mask))
        .map(|(_, archetype)| RefQueryChunk {
            len: archetype.len(),
            states: archetype.states().clone(),
            ptrs: L::prepare(archetype).unwrap(),
            mask,
        })
        .collect::<Vec<_>>();

    let len = chunks.iter().map(|chunk| chunk.len).sum();
    let last = chunks.pop();
    RefQueryIter {
        chunks,
        loaded: last,
        len,
        index: 0,
    }
}

// Mutable query that returns MutQueryItemResult
fn query_mut_raw<'a, L: MutQueryLayout<'a>>(archetypes: &mut ArchetypeSet) -> MutQueryIter<'a, L> {
    let access = L::access();
    let mask = access.shared() | access.unique();

    let mut chunks = archetypes
        .iter_mut()
        .filter(|(m, _)| m.contains(mask))
        .map(|(_, archetype)| MutQueryChunk {
            len: archetype.len(),
            states: archetype.states().clone(),
            ptrs: L::prepare(archetype).unwrap(),
            mask,
        })
        .collect::<Vec<_>>();

    let len = chunks.iter().map(|chunk| chunk.len).sum();
    let last = chunks.pop();
    MutQueryIter {
        chunks,
        loaded: last,
        len,
        access: L::access(),
        index: 0,
    }
}

// Immutable query that returns the layout tuple
pub(crate) fn query_ref<'a, L: RefQueryLayout<'a>>(archetypes: &ArchetypeSet) -> Option<impl Iterator<Item = L> + 'a> {
    L::is_valid().then(|| {
        query_ref_raw::<L>(archetypes).map(|item| item.tuple)
    })
}

// Mutable query that returns the layout tuple
pub(crate) fn query_mut<'a, L: MutQueryLayout<'a>>(archetypes: &mut ArchetypeSet) -> Option<impl Iterator<Item = L> + 'a> {
    L::is_valid().then(|| {
        query_mut_raw::<L>(archetypes).map(|item| item.tuple)
    })
}

// Immutable query that returns the layout tuple, filtered
pub(crate) fn query_ref_filter<'a, L: RefQueryLayout<'a>, E: Evaluate>(archetypes: &ArchetypeSet, filter: E) -> Option<impl Iterator<Item = L> + 'a> {
    L::is_valid().then(|| {
        let cached = E::prepare();
        query_ref_raw::<L>(archetypes).filter(move |item| E::eval(&cached, item.state, item.archetype_mask)).map(|item| item.tuple)
    })
}

// Mutable query that returns the layout tuple, filtered
pub(crate) fn query_mut_filter<'a, L: MutQueryLayout<'a>, E: Evaluate>(archetypes: &mut ArchetypeSet, filter: E) -> Option<impl Iterator<Item = L> + 'a> {
    L::is_valid().then(|| {
        let cached = E::prepare();
        query_mut_raw::<L>(archetypes).filter(move |item| E::eval(&cached, item.state, item.archetype_mask)).map(|item| item.tuple)
    })
}