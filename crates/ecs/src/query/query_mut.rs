use utils::BitSet;

use crate::{Always, Archetype, LayoutAccess, Mask, QueryFilter, QueryLayoutMut, Scene, Wrap};
use std::{iter::FusedIterator, marker::PhantomData};

/// This is a query that will be fetched from the main scene that we can use to get components out of entries with a specific layout.
/// Even though I define the 'it, 'b, and 's lfietimes, I don't use them in this query, I only use them in the query iterator.
pub struct QueryMut<'a: 'b, 'b, L: QueryLayoutMut> {
    pub(crate) archetypes: Vec<&'a mut Archetype>,
    access: LayoutAccess,
    bitsets: Option<Vec<BitSet<usize>>>,
    _phantom1: PhantomData<&'b ()>,
    _phantom3: PhantomData<L>,
}

impl<'a: 'b, 'b, L: QueryLayoutMut> QueryMut<'a, 'b, L> {
    // Create a new mut query from the scene
    pub(crate) fn new(scene: &'a mut Scene) -> Self {
        let (access, archetypes, _) = super::archetypes_mut::<L, Always>(scene.archetypes_mut());

        Self {
            archetypes,
            access,
            bitsets: None,
            _phantom1: PhantomData,
            _phantom3: PhantomData,
        }
    }

    // Create a new mut query from the scene, but make it have a specific entry enable/disable masks
    pub(crate) fn new_with_filter<F: QueryFilter>(
        scene: &'a mut Scene,
        _: Wrap<F>,
        ticked: bool,
    ) -> Self {
        // Filter out the archetypes then create the bitsets
        let (access, archetypes, cached) = super::archetypes_mut::<L, F>(scene.archetypes_mut());
        let bitsets =
            super::generate_bitset_chunks::<F>(archetypes.iter().map(|a| &**a), cached, ticked);

        Self {
            archetypes,
            access,
            bitsets: Some(bitsets),
            _phantom1: PhantomData,
            _phantom3: PhantomData,
        }
    }

    /// Get the access masks that we have calculated.
    pub fn layout_access(&self) -> LayoutAccess {
        self.access
    }

    /// Get the number of entries that we will have to iterate through.
    pub fn len(&self) -> usize {
        len(&self.archetypes, &self.bitsets)
    }

    /// Check if the query is empty.
    pub fn is_empty(&self) -> bool {
        self.archetypes.is_empty()
    }
}

// Update the mutability state column of a specific archetype based on a masks' compound unit masks
fn apply_mutability_states(
    archetype: &mut Archetype,
    mutability: Mask,
    bitset: Option<&BitSet<usize>>,
    ticked: bool,
) {
    let table = archetype.table_mut();
    for unit in mutability.units() {
        let column = table.get_mut(&unit).unwrap();
        let states = crate::get_either_states_mut(column, ticked);

        if let Some(bitset) = bitset {
            for (out_states, in_states) in
                states.chunks_mut().iter_mut().zip(bitset.chunks().iter())
            {
                out_states.modified = *in_states;
            }
        } else {
            for out in states.chunks_mut() {
                out.modified = usize::MAX;
            }
        }
    }
}

// Calculate the number of elements there are in the archetypes, but also take in consideration
// the bitsets (if specified)
fn len(archetypes: &[&mut Archetype], bitsets: &Option<Vec<BitSet<usize>>>) -> usize {
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

impl<'a: 'b, 'b, L: QueryLayoutMut> IntoIterator for QueryMut<'a, 'b, L> {
    type Item = L;
    type IntoIter = QueryMutIter<'b, L>;

    fn into_iter(mut self) -> Self::IntoIter {
        for (i, archetype) in self.archetypes.iter_mut().enumerate() {
            let bitset = self.bitsets.as_ref().map(|bitset| &bitset[i]);
            apply_mutability_states(
                archetype,
                archetype.mask() & self.access.unique(),
                bitset,
                false,
            );
            apply_mutability_states(
                archetype,
                archetype.mask() & self.access.unique(),
                bitset,
                true,
            );
        }

        QueryMutIter {
            archetypes: self.archetypes,
            bitsets: self.bitsets,
            chunk: None,
            index: 0,
            _phantom2: PhantomData,
        }
    }
}

// Currently loaded chunk in the mutable query iterator
struct Chunk<L: QueryLayoutMut> {
    bitset: Option<BitSet<usize>>,
    ptrs: L::PtrTuple,
    length: usize,
}

/// This is a mutable query iterator that will iterate through all the query entries in arbitrary order.
pub struct QueryMutIter<'b, L: QueryLayoutMut> {
    // Inputs from the query
    archetypes: Vec<&'b mut Archetype>,
    bitsets: Option<Vec<BitSet<usize>>>,

    // Unique to the iterator
    chunk: Option<Chunk<L>>,
    index: usize,
    _phantom2: PhantomData<L>,
}

impl<'b, L: QueryLayoutMut> QueryMutIter<'b, L> {
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
                bitset,
                ptrs,
                length,
            });
        }

        Some(())
    }
}

impl<'b, L: QueryLayoutMut> Iterator for QueryMutIter<'b, L> {
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
        let items = unsafe { L::read_mut_unchecked(ptrs, self.index) };
        self.index += 1;

        Some(items)
    }
}

impl<'b, L: QueryLayoutMut> ExactSizeIterator for QueryMutIter<'b, L> {}

impl<'b, 's, L: QueryLayoutMut> FusedIterator for QueryMutIter<'b, L> {}
