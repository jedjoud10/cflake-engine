use itertools::Itertools;
use smallvec::SmallVec;

use crate::{Archetype, Mask, QueryLayoutMut, Scene, StateRow, QueryFilter};
use std::marker::PhantomData;

// This is a query that will be fetched from the main scene that we can use to get components out of entries with a specific layout
// Even though I define the 'it, 'b, and 's lfietimes, I don't use them in this query, I only use them in the query iterator
pub struct QueryMut<'a: 'b, 'b, 's, L: for<'it> QueryLayoutMut<'it>> {
    archetypes: Vec<&'a mut Archetype>,
    mask: Mask,
    enabled: Option<Vec<u128>>,
    _phantom1: PhantomData<&'b ()>,
    _phantom2: PhantomData<&'s ()>,
    _phantom3: PhantomData<L>,
}

impl<'a: 'b, 'b, 's, L: for<'it> QueryLayoutMut<'it>> QueryMut<'a, 'b, 's, L> {
    // Get the archetypes and layout mask. Used internally only
    fn archetypes_mut(scene: &mut Scene) -> (Mask, Vec<&mut Archetype>) {
        let mask = L::reduce(|a, b| a | b).both();
        let archetypes = scene
            .archetypes_mut()
            .iter_mut()
            .filter_map(move |(&archetype_mask, archetype)| archetype_mask.contains(mask).then_some(archetype))
            .collect::<Vec<_>>();
        (mask, archetypes)
    }

    // Create a new mut query from the scene
    pub fn new(scene: &'a mut Scene) -> Self {
        let (mask, archetypes) = Self::archetypes_mut(scene);

        Self {
            archetypes,
            enabled: None,
            _phantom3: PhantomData,
            mask,
            _phantom1: PhantomData,
            _phantom2: PhantomData,
        }
    }

    // Create a new mut query from the scene, but make it have a specific entry enable/disable masks
    pub fn new_with_filter<F: QueryFilter>(scene: &'a mut Scene, filter: F) -> Self {
        let (mask, archetypes) = Self::archetypes_mut(scene);

        let cached = F::prepare();
        let mask = mask;
        /*
        let enabled = archetypes.iter().map(|archetype| {
            let states = archetype.states();
            let states = states.borrow();
            let iter = states.iter().cloned().map(|state| F::eval(&cached, state, mask));
            let chunks = iter.chunks(128);
            let chunks = chunks.into_iter();
            chunks.map(|chunk| chunk.fold(0, |accum, current| accum << 1 | (current as u128)))
        }).collect::<Vec<_>>();
        */

        Self {
            archetypes,
            mask,
            enabled: None,
            _phantom3: PhantomData,
            _phantom1: PhantomData,
            _phantom2: PhantomData,
        }
    }

    // Iterate through the query entries and execute a function for each one of them in another thread
    pub fn for_each(
        mut self,
        threadpool: &mut world::ThreadPool,
        function: impl Fn(<<L as QueryLayoutMut<'_>>::SliceTuple as world::SliceTuple<'_>>::ItemTuple) + Send + Sync + Clone,
        batch_size: usize,
    ) where for<'it, 's2>
        <L as QueryLayoutMut<'it>>::SliceTuple: world::SliceTuple<'s2>,
    {
        threadpool.scope(|scope| {
            for archetype in self.archetypes.iter_mut() {
                // Send the archetype slices to multiple threads to be able to compute them
                let ptrs = unsafe { L::ptrs_from_mut_archetype_unchecked(archetype) };
                let slices = unsafe { L::from_raw_parts(ptrs, archetype.len()) };
                scope.for_each(slices, function.clone(), batch_size);

                // We don't have to worry about doing this since the entry disabled/enabled mask is already computed when the query was created
                let mask = self.mask;
                let states = archetype.states();
                let mut borrowed = states.borrow_mut();
                
                // Update the mutable state masks
                for state in borrowed.iter_mut() {
                    StateRow::update(state, |_, _, mutated| *mutated = *mutated | mask);
                }
            }
        });
    }

    // Get the mask that we will use to filter through the archetypes
    pub fn mask(&self) -> Mask {
        self.mask
    }

    // Get the number of entries that we will have to iterate through
    pub fn len(&self) -> usize {
        self.archetypes.iter().map(|a| a.len()).sum()
    }
}

impl<'a: 'b, 'b, 'it, L: for<'s> QueryLayoutMut<'s>> IntoIterator for QueryMut<'a, 'b, 'it, L> {
    type Item = L;
    type IntoIter = QueryMutIter<'b, 'it, L>;

    fn into_iter(self) -> Self::IntoIter {
        QueryMutIter {
            archetypes: self.archetypes,
            chunk: None,
            index: 0,
            mask: self.mask,
            _phantom1: PhantomData,
            _phantom2: PhantomData,
        }
    }
}

// Currently loaded chunk in the mutable query iterator
struct Chunk<'b, 's, L: QueryLayoutMut<'s>> {
    archetype: &'b mut Archetype,
    ptrs: L::PtrTuple,
    length: usize,
}

// This is a mutable query iterator that will iterate through all the query entries in arbitrary order
pub struct QueryMutIter<'b, 's, L: QueryLayoutMut<'s>> {
    archetypes: Vec<&'b mut Archetype>,
    chunk: Option<Chunk<'b, 's, L>>,
    index: usize,
    mask: Mask,
    _phantom1: PhantomData<&'s ()>,
    _phantom2: PhantomData<L>,
}

impl<'b, 's, L: QueryLayoutMut<'s>> Iterator for QueryMutIter<'b, 's, L> {
    type Item = L;

    fn next(&mut self) -> Option<Self::Item> {
        // Hop onto the next archetype if we are done iterating through the current one
        if (self.index + 1) > self.chunk.as_ref().map(|chunk| chunk.length).unwrap_or_default() {
            let archetype = self.archetypes.pop()?;
            let ptrs = unsafe { L::ptrs_from_mut_archetype_unchecked(archetype) };
            let length = archetype.len();
            self.index = 0;
            self.chunk = Some(Chunk {
                archetype,
                ptrs,
                length,
            });
        }

        // I have to do this since iterators cannot return data that they are referencing, but in this case, it is safe to do so
        self.chunk.as_mut()?;
        let ptrs = self.chunk.as_ref().unwrap().ptrs;
        let items = unsafe { L::read_mut_unchecked(ptrs, 0) };
        self.index += 1;

        // Update the mask for the current entity
        let states = self.chunk.as_mut().unwrap().archetype.states();
        let mut vec = states.borrow_mut();
        vec[self.index].update(|_, _, update| *update = *update | self.mask);

        Some(items)
    }
}