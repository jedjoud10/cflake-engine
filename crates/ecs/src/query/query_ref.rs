use itertools::Itertools;
use math::BitSet;
use smallvec::SmallVec;

use crate::{
    Archetype, LayoutAccess, Mask, QueryFilter, QueryLayoutMut, QueryLayoutRef, Scene,
    Wrap, Always,
};
use std::{marker::PhantomData, sync::Arc, rc::Rc};

// This is a query that will be fetched from the main scene that we can use to get components out of entries with a specific layout
// Even though I define the 'it, 'b, and 's lfietimes, I don't use them in this query, I only use them in the query iterator
pub struct QueryRef<'a: 'b, 'b, 's, L: for<'it> QueryLayoutRef<'it>> {
    archetypes: Vec<&'a Archetype>,
    mask: Mask,
    bitsets: Option<Vec<BitSet>>,
    _phantom1: PhantomData<&'b ()>,
    _phantom2: PhantomData<&'s ()>,
    _phantom3: PhantomData<L>,
}

impl<'a: 'b, 'b, 's, L: for<'it> QueryLayoutRef<'it>> QueryRef<'a, 'b, 's, L> {
    // Create a new mut query from the scene
    pub fn new(scene: &'a Scene) -> Self {
        let (mask, archetypes, _) = super::archetypes::<L, Always>(scene);
        Self {
            archetypes,
            bitsets: None,
            _phantom3: PhantomData,
            mask,
            _phantom1: PhantomData,
            _phantom2: PhantomData,
        }
    }

    // Create a new mut query from the scene, but make it have a specific entry enable/disable masks
    pub fn new_with_filter<F: QueryFilter>(scene: &'a Scene, _: Wrap<F>) -> Self {
        // Filter out the archetypes then create the bitsets
        let (mask, archetypes, cached) = super::archetypes::<L, F>(scene);
        let bitsets = super::generate_bitset_chunks::<F>(archetypes.iter().map(|a| &**a), cached);

        Self {
            archetypes,
            mask,
            bitsets: Some(bitsets),
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
            for (i, archetype) in self.archetypes.iter().enumerate() {
                // Send the archetype slices to multiple threads to be able to compute them
                let ptrs = unsafe { L::ptrs_from_archetype_unchecked(archetype) };
                let slices = unsafe { L::from_raw_parts(ptrs, archetype.len()) };

                // Convert the archetype bitset to a thread-shareable bitset
                // TODO: Reverse the order of the archetypes to avoid cloning the bitset here
                let bitset = self.bitsets.as_ref().map(|bitset| Arc::new(bitset[i].clone()));

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
        if let Some(bitsets) = &self.bitsets {
            // TODO: Does this actually work
            bitsets.iter().zip(self.archetypes.iter()).map(|(b, a)| b.count_ones().min(a.len())).sum()
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
            bitsets: self.bitsets,
            chunk: None,
            index: 0,
            _phantom1: PhantomData,
            _phantom2: PhantomData,
        }
    }
}

// Currently loaded chunk in the immutable query iterator
struct Chunk<'s, L: QueryLayoutRef<'s>> {
    ptrs: L::PtrTuple,
    bitset: Option<BitSet>,
    length: usize,
}

// This is a immutable query iterator that will iterate through all the query entries in arbitrary order
pub struct QueryRefIter<'b, 's, L: QueryLayoutRef<'s>> {
    // Inputs from the query
    archetypes: Vec<&'b Archetype>,
    bitsets: Option<Vec<BitSet>>,

    // Unique to the iterator
    chunk: Option<Chunk<'s, L>>,
    index: usize,
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
        
        if self.index + 1 > len {
            let archetype = self.archetypes.pop()?;
            let bitset = self.bitsets.as_mut().map(|vec| vec.pop().unwrap());
            let ptrs = unsafe { L::ptrs_from_archetype_unchecked(archetype) };
            let length = archetype.len();
            self.index = 0;
            self.chunk = Some(Chunk {
                ptrs,
                bitset,
                length
            });
        }

        Some(())
    }
} 

impl<'b, 's, L: QueryLayoutRef<'s>> Iterator for QueryRefIter<'b, 's, L> {
    type Item = L;

    fn next(&mut self) -> Option<Self::Item> {
        loop {
            // Always hop to the next chunk at the start of the hop iteration / normal iteration 
            self.check_hop_chunk()?;

            if let Some(chunk) = &self.chunk {
                // Check for bitset
                if let Some(bitset) = &chunk.bitset {
                    // Check the next entry that is valid (that passed the filter)
                    if let Some(hop) = bitset.find_one_from(self.index) {
                        self.index = hop;
                        break;
                    } else {
                        // Hop to the next archetype if we could not find one
                        // This will force the iterator to hop to the next archetype
                        self.index = chunk.length;                        
                        continue;
                    }
            
                } else {
                    // If we do not have a bitset, don't do anything
                    break;
                }
            }
        }

        // I have to do this since iterators cannot return data that they are referencing, but in this case, it is safe to do so
        self.chunk.as_mut()?;
        let ptrs = self.chunk.as_ref().unwrap().ptrs;
        let items = unsafe { L::read_unchecked(ptrs, self.index) };
        self.index += 1;

        Some(items)
    }
}
