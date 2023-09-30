use std::{iter::FusedIterator, marker::PhantomData, ops::Deref};
use rayon::prelude::{IndexedParallelIterator, ParallelIterator};
use utils::bitset::BitSet;

use super::QueryMut;
use super::{Always, QueryFilter, Wrap, len};
use crate::{
    archetype::Archetype,
    layout::{LayoutAccess, QueryLayoutMut},
    mask::Mask,
    scene::Scene,
};

// Currently loaded chunk in the mutable query iterator
struct Chunk<L: QueryLayoutMut> {
    bitset: Option<BitSet<u64>>,
    ptrs: L::PtrTuple,
    length: usize,
}

/// This is a mutable query iterator that will iterate through all the query entries in arbitrary order.
pub struct QueryMutIter<'b, L: QueryLayoutMut> {
    // Inputs from the query
    archetypes: Vec<&'b mut Archetype>,
    bitsets: Option<Vec<BitSet<u64>>>,

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


impl<'a: 'b, 'b, L: QueryLayoutMut> IntoIterator for QueryMut<'a, 'b, L> {
    type Item = L;
    type IntoIter = QueryMutIter<'b, L>;

    fn into_iter(mut self) -> Self::IntoIter {
        /*
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
        */

        let archetype = self.archetypes.pop().unwrap();
        let bitset = self.bitsets.as_mut().map(|vec| vec.pop().unwrap());
        let ptrs = unsafe { L::ptrs_from_mut_archetype_unchecked(archetype) };
        let length = archetype.len();

        QueryMutIter {
            archetypes: self.archetypes,
            bitsets: self.bitsets,
            chunk: Some(Chunk {
                bitset,
                ptrs,
                length,
            }),
            index: 0,
            _phantom2: PhantomData,
        }
    }
}

impl<'b, L: QueryLayoutMut> Iterator for QueryMutIter<'b, L> {
    type Item = L;

    fn size_hint(&self) -> (usize, Option<usize>) {
        let len = len(&self.archetypes, &self.bitsets);
        (len, Some(len))
    }

    fn next(&mut self) -> Option<Self::Item> {
        let chunk = unsafe { self.chunk.as_ref().unwrap_unchecked() };
        if (self.index >= chunk.length) {
            return None;
        }

        /*
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
        */

        // I have to do this since iterators cannot return data that they are referencing, but in this case, it is safe to do so
        //self.chunk.as_mut()?;
        let ptrs = chunk.ptrs;
        let items = unsafe { L::read_mut_unchecked(ptrs, self.index) };
        self.index += 1;

        Some(items)
    }
}

impl<'b, L: QueryLayoutMut> ExactSizeIterator for QueryMutIter<'b, L> {}
impl<'b, 's, L: QueryLayoutMut> FusedIterator for QueryMutIter<'b, L> {}