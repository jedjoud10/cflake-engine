use itertools::Itertools;
use math::BitSet;
use smallvec::SmallVec;

use crate::{Archetype, LayoutAccess, Mask, QueryFilter, QueryLayoutMut, Scene, StateRow, Wrap};
use std::{marker::PhantomData, sync::Arc};

// This is a query that will be fetched from the main scene that we can use to get components out of entries with a specific layout
// Even though I define the 'it, 'b, and 's lfietimes, I don't use them in this query, I only use them in the query iterator
pub struct QueryMut<'a: 'b, 'b, 's, L: for<'it> QueryLayoutMut<'it>> {
    archetypes: Vec<&'a mut Archetype>,
    mask: Mask,
    mutability: Mask,
    bitset: Option<BitSet>,
    _phantom1: PhantomData<&'b ()>,
    _phantom2: PhantomData<&'s ()>,
    _phantom3: PhantomData<L>,
}

impl<'a: 'b, 'b, 's, L: for<'it> QueryLayoutMut<'it>> QueryMut<'a, 'b, 's, L> {
    // Get the archetypes and layout mask. Used internally only
    fn archetypes_mut(scene: &mut Scene) -> (LayoutAccess, Vec<&mut Archetype>) {
        let mask = L::reduce(|a, b| a | b);
        let archetypes = scene
            .archetypes_mut()
            .iter_mut()
            .filter_map(move |(&archetype_mask, archetype)| {
                archetype_mask.contains(mask.both()).then_some(archetype)
            })
            .collect::<Vec<_>>();
        (mask, archetypes)
    }

    // Create a new mut query from the scene
    pub fn new(scene: &'a mut Scene) -> Self {
        let (mask, archetypes) = Self::archetypes_mut(scene);
        let mutability = mask.unique();
        let mask = mask.both();

        Self {
            archetypes,
            bitset: None,
            _phantom3: PhantomData,
            mask,
            mutability,
            _phantom1: PhantomData,
            _phantom2: PhantomData,
        }
    }

    // Create a new mut query from the scene, but make it have a specific entry enable/disable masks
    pub fn new_with_filter<F: QueryFilter>(scene: &'a mut Scene, _: Wrap<F>) -> Self {
        let (mask, archetypes) = Self::archetypes_mut(scene);

        // Filter each archetype first
        let cached = F::prepare();
        let archetypes: Vec<&mut Archetype> = archetypes
            .into_iter()
            .filter(|a| F::eval_archetype(&cached, a))
            .collect();

        // Filter the entries by iterating the archetype state rows
        let mutability = mask.unique();
        let mask = mask.both();
        let iterator = archetypes.iter().flat_map(|archetype| {
            let states = archetype.states();
            states.iter().map(|state| F::eval_entry(&cached, *state))
        });
        let bitset = BitSet::from_iter(iterator);

        Self {
            archetypes,
            mask,
            mutability,
            bitset: Some(bitset),
            _phantom3: PhantomData,
            _phantom1: PhantomData,
            _phantom2: PhantomData,
        }
    }

    // Iterate through the query entries and execute a function for each one of them in another thread
    pub fn for_each(
        mut self,
        threadpool: &mut world::ThreadPool,
        function: impl Fn(<<L as QueryLayoutMut<'_>>::SliceTuple as world::SliceTuple<'_>>::ItemTuple)
            + Send
            + Sync
            + Clone,
        batch_size: usize,
    ) where
        for<'it, 's2> <L as QueryLayoutMut<'it>>::SliceTuple: world::SliceTuple<'s2>,
    {
        threadpool.scope(|scope| {
            let mutability = self.mutability;
            let bitset = self.bitset.map(|bitset| Arc::new(bitset));
            for archetype in self.archetypes.iter_mut() {
                // Send the archetype slices to multiple threads to be able to compute them
                let ptrs = unsafe { L::ptrs_from_mut_archetype_unchecked(archetype) };
                let slices = unsafe { L::from_raw_parts(ptrs, archetype.len()) };

                // Should we use per entry filtering?
                if let Some(bitset) = bitset.clone() {
                    scope.for_each_filtered(slices, function.clone(), bitset, batch_size);
                } else {
                    scope.for_each(slices, function.clone(), batch_size);
                }

                // We don't have to worry about doing this since the entry disabled/enabled mask is already computed when the query was created
                for state in archetype.states_mut().iter_mut() {
                    StateRow::update(state, |_, _, mutated| *mutated = *mutated | mutability);
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
            bitset: self.bitset,
            index: 0,
            mutability: self.mutability,
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
    mutability: Mask,
    bitset: Option<BitSet>,
    _phantom1: PhantomData<&'s ()>,
    _phantom2: PhantomData<L>,
}

impl<'b, 's, L: QueryLayoutMut<'s>> Iterator for QueryMutIter<'b, 's, L> {
    type Item = L;

    fn next(&mut self) -> Option<Self::Item> {
        // Hop onto the next archetype if we are done iterating through the current one
        if (self.index + 1)
            > self
                .chunk
                .as_ref()
                .map(|chunk| chunk.length)
                .unwrap_or_default()
        {
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

        // Skip the archetype if we are using a filter
        if let Some(bitset) = &self.bitset {
            if !bitset.get(self.index) {
                self.index += 1;
                return None;
            }
        }

        // I have to do this since iterators cannot return data that they are referencing, but in this case, it is safe to do so
        self.chunk.as_mut()?;
        let ptrs = self.chunk.as_ref().unwrap().ptrs;
        let items = unsafe { L::read_mut_unchecked(ptrs, self.index) };
        self.index += 1;

        // Update the mask for the current entity
        let states = self.chunk.as_mut().unwrap().archetype.states_mut();
        states[self.index - 1].update(|_, _, update| *update = *update | self.mutability);

        Some(items)
    }
}
