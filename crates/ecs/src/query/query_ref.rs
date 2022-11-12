use itertools::Itertools;
use math::BitSet;
use smallvec::SmallVec;

use crate::{
    Archetype, LayoutAccess, Mask, QueryFilter, QueryLayoutMut, QueryLayoutRef, Scene, StateRow,
    Wrap,
};
use std::{marker::PhantomData, sync::Arc, rc::Rc};

// This is a query that will be fetched from the main scene that we can use to get components out of entries with a specific layout
// Even though I define the 'it, 'b, and 's lfietimes, I don't use them in this query, I only use them in the query iterator
pub struct QueryRef<'a: 'b, 'b, 's, L: for<'it> QueryLayoutRef<'it>> {
    archetypes: Vec<&'a Archetype>,
    mask: Mask,
    bitset: Option<BitSet>,
    _phantom1: PhantomData<&'b ()>,
    _phantom2: PhantomData<&'s ()>,
    _phantom3: PhantomData<L>,
}

impl<'a: 'b, 'b, 's, L: for<'it> QueryLayoutRef<'it>> QueryRef<'a, 'b, 's, L> {
    // Get the archetypes and layout mask. Used internally only
    fn archetypes(scene: &Scene) -> (LayoutAccess, Vec<&Archetype>) {
        let mask = L::reduce(|a, b| a | b);
        let archetypes = scene
            .archetypes()
            .iter()
            .filter_map(move |(&archetype_mask, archetype)| {
                (archetype.len() > 0 && archetype_mask.contains(mask.both())).then_some(archetype)
            })
            .collect::<Vec<_>>();
        (mask, archetypes)
    }

    // Create a new mut query from the scene
    pub fn new(scene: &'a Scene) -> Self {
        let (mask, archetypes) = Self::archetypes(scene);
        let mask = mask.both();

        Self {
            archetypes,
            bitset: None,
            _phantom3: PhantomData,
            mask,
            _phantom1: PhantomData,
            _phantom2: PhantomData,
        }
    }

    // Create a new mut query from the scene, but make it have a specific entry enable/disable masks
    pub fn new_with_filter<F: QueryFilter>(scene: &'a Scene, _: Wrap<F>) -> Self {
        let (mask, archetypes) = Self::archetypes(scene);

        // Filter each archetype first
        let cached = F::prepare();
        let archetypes: Vec<&Archetype> = archetypes
            .into_iter()
            .filter(|a| F::eval_archetype(&cached, a))
            .collect();

        // Filter the entries by iterating the archetype state rows
        let iterator = archetypes.iter().flat_map(|archetype| {
            let states = archetype.states();
            states.iter().map(|state| F::eval_entry(&cached, *state))
        });
        let bitset = BitSet::from_iter(iterator);

        Self {
            archetypes,
            mask: mask.both(),
            bitset: Some(bitset),
            _phantom3: PhantomData,
            _phantom1: PhantomData,
            _phantom2: PhantomData,
        }
    }

    // Iterate through the query entries and execute a function for each one of them in another thread
    pub fn for_each(
        self,
        threadpool: &mut world::ThreadPool,
        function: impl Fn(<<L as QueryLayoutRef<'_>>::SliceTuple as world::SliceTuple<'_>>::ItemTuple)
            + Send
            + Sync
            + Clone,
        batch_size: usize,
    ) where
        for<'it, 's2> <L as QueryLayoutRef<'it>>::SliceTuple: world::SliceTuple<'s2>,
    {
        threadpool.scope(|scope| {
            let bitset = self.bitset.map(|bitset| Arc::new(bitset));
            for archetype in self.archetypes.iter() {
                // Send the archetype slices to multiple threads to be able to compute them
                let ptrs = unsafe { L::ptrs_from_archetype_unchecked(archetype) };
                let slices = unsafe { L::from_raw_parts(ptrs, archetype.len()) };

                // Should we use per entry filtering?
                if let Some(bitset) = bitset.clone() {
                    scope.for_each_filtered(slices, function.clone(), bitset, batch_size);
                } else {
                    scope.for_each(slices, function.clone(), batch_size);
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
        if let Some(bitset) = &self.bitset {
            bitset.count_ones()
        } else {
            self.archetypes.iter().map(|a| a.len()).sum()
        }
    }
}

impl<'a: 'b, 'b, 'it, L: for<'s> QueryLayoutRef<'s>> IntoIterator for QueryRef<'a, 'b, 'it, L> {
    type Item = L;
    type IntoIter = QueryRefIter<'b, 'it, L>;

    fn into_iter(self) -> Self::IntoIter {
        QueryRefIter {
            archetypes: self.archetypes,
            chunk: None,
            bitset: self.bitset.map(Rc::new),
            local_index: 0,
            global_index: 0,
            _phantom1: PhantomData,
            _phantom2: PhantomData,
        }
    }
}

// Currently loaded chunk in the immutable query iterator
struct Chunk<'s, L: QueryLayoutRef<'s>> {
    ptrs: L::PtrTuple,
    length: usize,
}

// This is a immutable query iterator that will iterate through all the query entries in arbitrary order
pub struct QueryRefIter<'b, 's, L: QueryLayoutRef<'s>> {
    archetypes: Vec<&'b Archetype>,
    chunk: Option<Chunk<'s, L>>,
    local_index: usize,
    global_index: usize,
    bitset: Option<Rc<BitSet>>,
    _phantom1: PhantomData<&'s ()>,
    _phantom2: PhantomData<L>,
}

impl<'b, 's, L: QueryLayoutRef<'s>> QueryRefIter<'b, 's, L> {
    // Hop onto the next archetype if we are done iterating through the current one
    fn check_hop_chunk(&mut self) -> Option<()> {
        let len = self
            .chunk
            .as_ref()
            .map(|chunk| chunk.length)
            .unwrap_or_default();
        
        if self.local_index + 1 > len {
            let archetype = self.archetypes.pop()?;
            let ptrs = unsafe { L::ptrs_from_archetype_unchecked(archetype) };
            let length = archetype.len();
            self.local_index = 0;
            self.chunk = Some(Chunk { ptrs, length });
        }

        Some(())
    }
} 

impl<'b, 's, L: QueryLayoutRef<'s>> Iterator for QueryRefIter<'b, 's, L> {
    type Item = L;

    fn next(&mut self) -> Option<Self::Item> {
        // Check if we should hop chunks
        self.check_hop_chunk()?;

        // Skip the archetype if we are using a filter
        if let Some(bitset) = self.bitset.clone() {
            // Increment the local index and global index until we find a set bit
            dbg!(self.global_index);
            dbg!(self.local_index);
            let mut bit = bitset.get(self.global_index);
            dbg!(bit);
            while !bit {
                self.local_index += 1;
                self.global_index += 1;
                dbg!(self.chunk.as_ref().unwrap().length);
                self.check_hop_chunk()?;
                dbg!(self.chunk.as_ref().unwrap().length);
                bit = bitset.get(self.global_index);
                dbg!(bit);
            }
        }

        // I have to do this since iterators cannot return data that they are referencing, but in this case, it is safe to do so
        self.chunk.as_mut()?;
        let ptrs = self.chunk.as_ref().unwrap().ptrs;
        let items = unsafe { L::read_unchecked(ptrs, self.local_index) };
        self.local_index += 1;
        self.global_index += 1;

        Some(items)
    }
}
