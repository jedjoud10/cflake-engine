use utils::bitset::BitSet;

use std::{iter::FusedIterator, marker::PhantomData};

use crate::{
    archetype::Archetype,
    layout::{LayoutAccess, QueryLayoutRef},
    scene::Scene,
};

use super::{Always, QueryFilter, Wrap, len, QueryRef};

// Currently loaded chunk in the immutable query iterator
struct Chunk<'s, L: QueryLayoutRef<'s>> {
    slices: L::SliceTuple,
    bitset: Option<BitSet<u64>>,
    length: usize,
}

/// This is a immutable query iterator that will iterate through all the query entries in arbitrary order.
pub struct QueryRefIter<'s, L: QueryLayoutRef<'s>> {
    slices: Vec<Chunk<'s, L>>,
    chunk: Option<Chunk<'s, L>>,
    index: usize,
    length: usize,
    _phantom: PhantomData<L>,
}

impl<'s, L: QueryLayoutRef<'s>> IntoIterator for QueryRef<'s, L> {
    type Item = L;
    type IntoIter = QueryRefIter<'s, L>;

    fn into_iter(mut self) -> Self::IntoIter {
        let length = len(&self.archetypes);
        let mut slices = self.archetypes.into_iter().enumerate().map(|(index, archetype)| Chunk {
            slices: L::from_archetype(archetype),
            bitset: self.bitsets.as_mut().map(|vec| std::mem::replace(&mut vec[index], BitSet::<u64>::default())),
            length: archetype.len(),
        }).collect::<Vec<_>>();
        
        QueryRefIter {
            chunk: slices.pop(),
            slices,
            index: 0,
            length,
            _phantom: PhantomData,
        }
    }
}

impl<'s, L: QueryLayoutRef<'s>> Iterator for QueryRefIter<'s, L> {
    type Item = L;

    fn size_hint(&self) -> (usize, Option<usize>) {
        (self.length, Some(self.length))
    }

    fn next(&mut self) -> Option<Self::Item> {
        loop {
            if self.index >= self.chunk.as_ref()?.length {
                let chunk = self.slices.pop()?;
                self.chunk = Some(chunk);
                self.index = 0;
            }

            if let Some(chunk) = &self.chunk {
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

        let chunk = unsafe { self.chunk.as_ref().unwrap_unchecked() };
        let out = L::read(chunk.slices, self.index);
        self.index += 1;
        Some(out)
    }
}

impl<'s, L: QueryLayoutRef<'s>> ExactSizeIterator for QueryRefIter<'s, L> {}
impl<'s, L: QueryLayoutRef<'s>> FusedIterator for QueryRefIter<'s, L> {}
