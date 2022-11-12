use itertools::Itertools;
use math::BitSet;
use smallvec::SmallVec;
use crate::{Archetype, LayoutAccess, Mask, QueryFilter, QueryLayoutMut, Scene, Wrap};
use std::{marker::PhantomData, sync::Arc, rc::Rc};

// This is a query that will be fetched from the main scene that we can use to get components out of entries with a specific layout
// Even though I define the 'it, 'b, and 's lfietimes, I don't use them in this query, I only use them in the query iterator
pub struct QueryMut<'a: 'b, 'b, 's, L: for<'it> QueryLayoutMut<'it>> {
    archetypes: Vec<&'a mut Archetype>,
    mask: Mask,
    mutability: Mask,
    bitsets: Option<Vec<BitSet>>,
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
            bitsets: None,
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
            .filter(|a| F::evaluate_archetype(cached, a))
            .collect();

        // Filter the entries by iterating the archetype state rows
        let mutability = mask.unique();
        let mask = mask.both();


        // Filter the entries by chunks of 64 entries at a time
        let iterator = archetypes.iter().map(|archetype| {
            let columns = F::cache_columns(cached, archetype);
            let chunks = archetype.entities().len() as f32 / usize::BITS as f32;
            let chunks = chunks.ceil() as usize;
            BitSet::from_chunks_iter((0..chunks).into_iter().map(move |i| 
                F::evaluate_chunk(columns, i)
            ))
        });

        // Create a unique hop bitset for each archetype
        let bitsets = Vec::from_iter(iterator);

        Self {
            archetypes,
            mask,
            mutability,
            bitsets: Some(bitsets),
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
            for (i, archetype) in self.archetypes.iter_mut().enumerate() {
                // Send the archetype slices to multiple threads to be able to compute them
                let ptrs = unsafe { L::ptrs_from_mut_archetype_unchecked(archetype) };
                let slices = unsafe { L::from_raw_parts(ptrs, archetype.len()) };

                // Convert the archetype bitset to a thread-shareable bitset
                // TODO: Reverse the order of the archetypes to avoid cloning the bitset here
                let bitset = self.bitsets.as_ref().map(|bitset| Arc::new(bitset[i].clone()));

                // Update all states chunks of the current archetype before iterating
                let table = archetype.state_table_mut();
                for mask in mutability.units() {
                    let column = table.get_mut(&mask).unwrap();
                    
                    // Update all the chunks that are stored in the column
                    if let Some(bitset) = &bitset {
                        let chunks = bitset.chunks();
                        for (i, output) in column.chunks_mut().iter_mut().enumerate() {
                            output.modified = chunks[i];
                        }
                    } else {
                        for chunk in column.chunks_mut().iter_mut() {
                            chunk.modified = usize::MAX;
                        }
                    }
                } 

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
            bitsets.iter().map(|b| b.count_ones()).sum()
        } else {
            self.archetypes.iter().map(|a| a.len()).sum()
        }
    }
}

impl<'a: 'b, 'b, 'it, L: for<'s> QueryLayoutMut<'s>> IntoIterator for QueryMut<'a, 'b, 'it, L> {
    type Item = L;
    type IntoIter = QueryMutIter<'b, 'it, L>;

    fn into_iter(self) -> Self::IntoIter {
        // TODO: Update all states chunks of the current archetype before iterating

        QueryMutIter {
            archetypes: self.archetypes,
            bitsets: self.bitsets,
            chunk: None,
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
    bitset: Option<BitSet>,
    ptrs: L::PtrTuple,
    length: usize,
}

// This is a mutable query iterator that will iterate through all the query entries in arbitrary order
pub struct QueryMutIter<'b, 's, L: QueryLayoutMut<'s>> {
    // Inputs from the query
    archetypes: Vec<&'b mut Archetype>,
    bitsets: Option<Vec<BitSet>>,

    // Unique to the iterator
    chunk: Option<Chunk<'b, 's, L>>,
    index: usize,
    mutability: Mask,
    _phantom1: PhantomData<&'s ()>,
    _phantom2: PhantomData<L>,
}


impl<'b, 's, L: QueryLayoutMut<'s>> QueryMutIter<'b, 's, L> {
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
            let ptrs = unsafe { L::ptrs_from_mut_archetype_unchecked(archetype) };
            let length = archetype.len();
            self.index = 0;
            self.chunk = Some(Chunk {
                archetype,
                bitset,
                ptrs,
                length,
            });
        }

        Some(())
    }
} 

impl<'b, 's, L: QueryLayoutMut<'s>> Iterator for QueryMutIter<'b, 's, L> {
    type Item = L;

    fn next(&mut self) -> Option<Self::Item> {
        self.check_hop_chunk()?;

        // Skip the archetype if we are using a filter
        if let Some(chunk) = &self.chunk {
            if let Some(bitset) = &chunk.bitset {
                // TODO: Implement bitset hopping
            }
        }

        // I have to do this since iterators cannot return data that they are referencing, but in this case, it is safe to do so
        self.chunk.as_mut()?;
        let ptrs = self.chunk.as_ref().unwrap().ptrs;
        let items = unsafe { L::read_mut_unchecked(ptrs, self.index) };
        self.index += 1;

        Some(items)
    }
}
