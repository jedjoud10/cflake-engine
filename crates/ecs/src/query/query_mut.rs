use utils::BitSet;

use crate::{
    Always, Archetype, LayoutAccess, Mask, QueryFilter,
    QueryLayoutMut, Scene, Wrap,
};
use std::{iter::FusedIterator, marker::PhantomData};

// This is a query that will be fetched from the main scene that we can use to get components out of entries with a specific layout
// Even though I define the 'it, 'b, and 's lfietimes, I don't use them in this query, I only use them in the query iterator
pub struct QueryMut<'a: 'b, 'b, 's, L: for<'it> QueryLayoutMut<'it>> {
    pub(crate) archetypes: Vec<&'a mut Archetype>,
    access: LayoutAccess,
    bitsets: Option<Vec<BitSet>>,
    _phantom1: PhantomData<&'b ()>,
    _phantom2: PhantomData<&'s ()>,
    _phantom3: PhantomData<L>,
}

impl<'a: 'b, 'b, 's, L: for<'it> QueryLayoutMut<'it>>
    QueryMut<'a, 'b, 's, L>
{
    // Create a new mut query from the scene
    pub fn new(scene: &'a mut Scene) -> Self {
        let (access, archetypes, _) =
            super::archetypes_mut::<L, Always>(scene.archetypes_mut());

        Self {
            archetypes,
            bitsets: None,
            _phantom3: PhantomData,
            access,
            _phantom1: PhantomData,
            _phantom2: PhantomData,
        }
    }

    // Create a new mut query from the scene, but make it have a specific entry enable/disable masks
    pub fn new_with_filter<F: QueryFilter>(
        scene: &'a mut Scene,
        _: Wrap<F>,
    ) -> Self {
        // Filter out the archetypes then create the bitsets
        let (access, archetypes, cached) =
            super::archetypes_mut::<L, F>(scene.archetypes_mut());
        let bitsets = super::generate_bitset_chunks::<F>(
            archetypes.iter().map(|a| &**a),
            cached,
        );

        Self {
            archetypes,
            access,
            bitsets: Some(bitsets),
            _phantom3: PhantomData,
            _phantom1: PhantomData,
            _phantom2: PhantomData,
        }
    }

    // Iterate through the query entries and execute a function for each one of them in another thread
    pub fn for_each(
        mut self,
        threadpool: &mut utils::ThreadPool,
        function: impl Fn(<<L as QueryLayoutMut<'_>>::SliceTuple as utils::SliceTuple<'_>>::ItemTuple)
            + Send
            + Sync
            + Clone,
        batch_size: usize,
    ) where
        for<'it, 's2> <L as QueryLayoutMut<'it>>::SliceTuple:
            utils::SliceTuple<'s2>,
    {
        threadpool.scope(|scope| {
            // Convert the optional bitset vector to an iterator that returns None if it is None
            let bitsets = self
                .bitsets
                .into_iter()
                .flatten()
                .map(|b| Some(b))
                .chain(std::iter::repeat(None));
            for (archetype, bitset) in
                self.archetypes.iter_mut().zip(bitsets)
            {
                // Send the archetype slices to multiple threads to be able to compute them
                let ptrs = unsafe {
                    L::ptrs_from_mut_archetype_unchecked(archetype)
                };
                let slices = unsafe {
                    L::from_raw_parts(ptrs, archetype.len())
                };

                // Update all states chunks of the current archetype before iterating
                apply_mutability_states(
                    archetype,
                    archetype.mask() & self.access.unique(),
                    bitset.as_ref(),
                );

                // Should we use per entry filtering?
                if let Some(bitset) = bitset {
                    scope.for_each_filtered(
                        slices,
                        function.clone(),
                        bitset,
                        batch_size,
                    );
                } else {
                    scope.for_each(
                        slices,
                        function.clone(),
                        batch_size,
                    );
                }
            }
        });
    }

    // Get the access masks that we have calculated
    pub fn layout_access(&self) -> LayoutAccess {
        self.access
    }

    // Get the number of entries that we will have to iterate through
    pub fn len(&self) -> usize {
        len(&self.archetypes, &self.bitsets)
    }

    // Check if the query is empty
    pub fn is_empty(&self) -> bool {
        self.archetypes.is_empty()
    }
}

// Update the mutability state column of a specific archetype based on a masks' compound unit masks
fn apply_mutability_states(
    archetype: &mut Archetype,
    mutability: Mask,
    bitset: Option<&BitSet>,
) {
    let table = archetype.state_table_mut();
    for unit in mutability.units() {
        let column = table.get_mut(&unit).unwrap();

        if let Some(bitset) = bitset {
            for (out_states, in_states) in column
                .chunks_mut()
                .iter_mut()
                .zip(bitset.chunks().iter())
            {
                out_states.modified = *in_states;
            }
        } else {
            for out in column.chunks_mut() {
                out.modified = usize::MAX;
            }
        }
    }
}

// Calculate the number of elements there are in the archetypes, but also take in consideration
// the bitsets (if specified)
fn len(
    archetypes: &[&mut Archetype],
    bitsets: &Option<Vec<BitSet>>,
) -> usize {
    if let Some(bitsets) = bitsets {
        bitsets
            .iter()
            .zip(archetypes.iter())
            .map(|(b, a)| b.count_ones().min(a.len()))
            .sum()
    } else {
        archetypes.iter().map(|a| a.len()).sum()
    }
}

impl<'a: 'b, 'b, 'it, L: for<'s> QueryLayoutMut<'s>> IntoIterator
    for QueryMut<'a, 'b, 'it, L>
{
    type Item = L;
    type IntoIter = QueryMutIter<'b, 'it, L>;

    fn into_iter(mut self) -> Self::IntoIter {
        for (i, archetype) in self.archetypes.iter_mut().enumerate() {
            let bitset =
                self.bitsets.as_ref().map(|bitset| &bitset[i]);
            apply_mutability_states(
                archetype,
                archetype.mask() & self.access.unique(),
                bitset,
            );
        }

        QueryMutIter {
            archetypes: self.archetypes,
            bitsets: self.bitsets,
            chunk: None,
            index: 0,
            _phantom1: PhantomData,
            _phantom2: PhantomData,
        }
    }
}

// Currently loaded chunk in the mutable query iterator
struct Chunk<'b, 's, L: QueryLayoutMut<'s>> {
    _phantom: PhantomData<&'b ()>,
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
            let bitset =
                self.bitsets.as_mut().map(|vec| vec.pop().unwrap());
            let ptrs = unsafe {
                L::ptrs_from_mut_archetype_unchecked(archetype)
            };
            let length = archetype.len();
            self.index = 0;
            self.chunk = Some(Chunk {
                _phantom: PhantomData,
                bitset,
                ptrs,
                length,
            });
        }

        Some(())
    }
}

impl<'b, 's, L: QueryLayoutMut<'s>> Iterator
    for QueryMutIter<'b, 's, L>
{
    type Item = L;

    fn size_hint(&self) -> (usize, Option<usize>) {
        let len = len(&self.archetypes, &self.bitsets);
        (len, Some(len))
    }

    fn next(&mut self) -> Option<Self::Item> {
        loop {
            // Always hop to the next chunk at the start of the hop iteration / normal iteration
            self.check_hop_chunk()?;

            if let Some(chunk) = &self.chunk {
                // Check for bitset
                if let Some(bitset) = &chunk.bitset {
                    // Check the next entry that is valid (that passed the filter)
                    if let Some(hop) =
                        bitset.find_one_from(self.index)
                    {
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
        let items =
            unsafe { L::read_mut_unchecked(ptrs, self.index) };
        self.index += 1;

        Some(items)
    }
}

impl<'b, 's, L: QueryLayoutMut<'s>> ExactSizeIterator
    for QueryMutIter<'b, 's, L>
{
}

impl<'b, 's, L: QueryLayoutMut<'s>> FusedIterator
    for QueryMutIter<'b, 's, L>
{
}
