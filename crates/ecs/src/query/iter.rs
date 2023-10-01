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

    fn into_iter(self) -> Self::IntoIter {
        let length = len(&self.archetypes);
        let mut slices = self.archetypes.into_iter().map(|x| Chunk {
            slices: L::from_archetype(x),
            length: x.len(),
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
        if self.index >= self.chunk.as_ref()?.length {
            let chunk = self.slices.pop()?;
            self.chunk = Some(chunk);
            self.index = 0;
        }

        let chunk = unsafe { self.chunk.as_ref().unwrap_unchecked() };
        let out = L::read(chunk.slices, self.index);
        self.index += 1;
        Some(out)
    }
}

impl<'s, L: QueryLayoutRef<'s>> ExactSizeIterator for QueryRefIter<'s, L> {}
impl<'s, L: QueryLayoutRef<'s>> FusedIterator for QueryRefIter<'s, L> {}
