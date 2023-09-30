use utils::bitset::BitSet;

use std::{iter::FusedIterator, marker::PhantomData};

use crate::{
    archetype::Archetype,
    layout::{LayoutAccess, QueryLayoutRef},
    scene::Scene,
};

use super::{Always, QueryFilter, Wrap, len, QueryRef};

// Currently loaded chunk in the immutable query iterator
struct Chunk<'a, L: QueryLayoutRef> {
    slice: L::SliceTuple<'a>,
    length: usize,
}

/// This is a immutable query iterator that will iterate through all the query entries in arbitrary order.
pub struct QueryRefIter<'b, L: QueryLayoutRef> {
    // Inputs from the query
    slices: Vec<Chunk<'b, L>>,

    // Unique to the iterator
    chunk: Option<Chunk<'b, L>>,
    index: usize,
    length: usize,
    _phantom2: PhantomData<L>,
}

impl<'a: 'b, 'b, 'it, L: QueryLayoutRef> IntoIterator for QueryRef<'a, 'b, 'it, L> {
    type Item = L;
    type IntoIter = QueryRefIter<'b, L>;

    fn into_iter(self) -> Self::IntoIter {
        let length = len(&self.archetypes);
        let slices = self.archetypes.into_iter().map(|x| Chunk {
            slice: L::from_archetype(x),
            length: x.len(),
        }).collect();
        
        QueryRefIter {
            slices,
            chunk: None,
            index: 0,
            length,
            _phantom2: PhantomData,
        }
    }
}

impl<'b, L: QueryLayoutRef> Iterator for QueryRefIter<'b, L> {
    type Item = L;

    fn size_hint(&self) -> (usize, Option<usize>) {
        (self.length, Some(self.length))
    }

    fn next(&mut self) -> Option<Self::Item> {
        todo!()
        /*
        let chunk = self.chunk.as_ref()?;

        if self.index >= chunk.length {
            let slice = self.slices.pop()?;
            let ptrs = unsafe { L::ptrs_from_archetype_unchecked(archetype) };
            let length = archetype.len();
            let slice = unsafe { L::from_raw_parts(ptrs, length) };
            self.index = 0;
            self.chunk = Some(Chunk {
                slice,
                length,
            });
        }

        let items = unsafe { L::read(ptrs, self.index) };
        self.index += 1;

        Some(items)
        */
    }
}

impl<'b, L: QueryLayoutRef> ExactSizeIterator for QueryRefIter<'b, L> {}
impl<'b, 's, L: QueryLayoutRef> FusedIterator for QueryRefIter<'b, L> {}
