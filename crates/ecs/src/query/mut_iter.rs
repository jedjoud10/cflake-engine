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
struct Chunk<'s, L: QueryLayoutMut<'s>> {
    slices: L::SliceTuple,
    bitset: Option<BitSet<u64>>,
    length: usize,
}

/// This is a mutable query iterator that will iterate through all the query entries in arbitrary order.
pub struct QueryMutIter<'s, L: QueryLayoutMut<'s>> {
    slices: Vec<Chunk<'s, L>>,
    chunk: Option<Chunk<'s, L>>,
    index: usize,
    length: usize,
    _phantom: PhantomData<L>,
}

impl<'s, L: QueryLayoutMut<'s>> IntoIterator for QueryMut<'s, L> {
    type Item = L;
    type IntoIter = QueryMutIter<'s, L>;

    fn into_iter(mut self) -> Self::IntoIter {
        let length = len(&self.archetypes);
        
        for (index, archetype) in self.archetypes.iter_mut().enumerate() {
            super::utils::apply_mutability_states(
                archetype,
                archetype.mask() & self.access.unique(),
                self.bitsets.as_mut().map(|vec| &vec[index]),
            );
        }
        
        let mut slices = self.archetypes.into_iter().enumerate().map(|(index, archetype)| Chunk {
            length: archetype.len(),
            slices: L::from_mut_archetype(archetype),
            bitset: self.bitsets.as_mut().map(|vec| std::mem::replace(&mut vec[index], BitSet::<u64>::default())),
        }).collect::<Vec<_>>();
        
        QueryMutIter {
            chunk: slices.pop(),
            slices,
            index: 0,
            length,
            _phantom: PhantomData,
        }
    }
}

impl<'s, L: QueryLayoutMut<'s>> Iterator for QueryMutIter<'s, L> {
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

        let chunk = unsafe { self.chunk.as_mut().unwrap_unchecked() };

        // Iterators don't allow returning internal references, so we can cheat a bit
        // NOTE: There is most definitely a way to make this safe but I don't know how exactly
        // TODO: Not important (cause it works), but it's a hack. Pls fix
        let val = &mut chunk.slices;
        let copied = unsafe { std::mem::transmute_copy::<_, L::SliceTuple>(val) };
        let out = L::read_mut(copied, self.index);
        self.index += 1;
        Some(out)
    }
}

impl<'s, L: QueryLayoutMut<'s>> ExactSizeIterator for QueryMutIter<'s, L> {}
impl<'s, L: QueryLayoutMut<'s>> FusedIterator for QueryMutIter<'s, L> {}