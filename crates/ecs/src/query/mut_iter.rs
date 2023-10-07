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
        let mut slices = self.archetypes.into_iter().map(|x| Chunk {
            length: x.len(),
            slices: L::from_mut_archetype(x),
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
        if self.index >= self.chunk.as_ref()?.length {
            let chunk = self.slices.pop()?;
            self.chunk = Some(chunk);
            self.index = 0;
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