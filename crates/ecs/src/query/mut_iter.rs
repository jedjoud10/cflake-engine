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
        todo!()
    }
}

impl<'b, L: QueryLayoutMut> Iterator for QueryMutIter<'b, L> {
    type Item = L;

    fn size_hint(&self) -> (usize, Option<usize>) {
        let len = len(&self.archetypes);
        (len, Some(len))
    }

    fn next(&mut self) -> Option<Self::Item> {
        None
    }
}

impl<'b, L: QueryLayoutMut> ExactSizeIterator for QueryMutIter<'b, L> {}
impl<'b, 's, L: QueryLayoutMut> FusedIterator for QueryMutIter<'b, L> {}