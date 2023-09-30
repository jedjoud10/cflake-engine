use utils::bitset::BitSet;

use std::{iter::FusedIterator, marker::PhantomData};

use crate::{
    archetype::Archetype,
    layout::{LayoutAccess, QueryLayoutRef},
    scene::Scene,
};

use super::{Always, QueryFilter, Wrap, len, QueryRef};

// Currently loaded chunk in the immutable query iterator
struct Chunk<L: QueryLayoutRef> {
    ptrs: L::PtrTuple,
    bitset: Option<BitSet<u64>>,
    length: usize,
}

/// This is a immutable query iterator that will iterate through all the query entries in arbitrary order.
pub struct QueryRefIter<'b, L: QueryLayoutRef> {
    // Inputs from the query
    archetypes: Vec<&'b Archetype>,
    bitsets: Option<Vec<BitSet<u64>>>,

    // Unique to the iterator
    chunk: Option<Chunk<L>>,
    index: usize,
    _phantom2: PhantomData<L>,
}

impl<'b, 's, L: QueryLayoutRef> QueryRefIter<'b, L> {
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
                length,
            });
        }

        Some(())
    }
}

impl<'a: 'b, 'b, 'it, L: QueryLayoutRef> IntoIterator for QueryRef<'a, 'b, 'it, L> {
    type Item = L;
    type IntoIter = QueryRefIter<'b, L>;

    fn into_iter(self) -> Self::IntoIter {
        QueryRefIter {
            archetypes: self.archetypes,
            bitsets: self.bitsets,
            chunk: None,
            index: 0,
            _phantom2: PhantomData,
        }
    }
}

impl<'b, L: QueryLayoutRef> Iterator for QueryRefIter<'b, L> {
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
        let items = unsafe { L::read_unchecked(ptrs, self.index) };
        self.index += 1;

        Some(items)
    }
}

impl<'b, L: QueryLayoutRef> ExactSizeIterator for QueryRefIter<'b, L> {}
impl<'b, 's, L: QueryLayoutRef> FusedIterator for QueryRefIter<'b, L> {}
