use std::{iter::FusedIterator, marker::PhantomData, ops::Deref};
use rayon::iter::plumbing::bridge;
use rayon::prelude::{IndexedParallelIterator, ParallelIterator, IntoParallelIterator};
use utils::bitset::BitSet;

use super::QueryMut;
use super::{Always, QueryFilter, Wrap, len};
use crate::{
    archetype::Archetype,
    layout::{LayoutAccess, QueryLayoutMut},
    mask::Mask,
    scene::Scene,
};

/// Mutable parallel iterator that makes use of rayon's thread pool for multithreading
/// This must be able to "split" off 
pub struct QueryMutParIter<'b, L: QueryLayoutMut> {
    index: usize,
    _phantom2: PhantomData<&'b L>,
}

/*
struct QueryMutParProducer<'b, L: QueryLayoutMut> {

}
*/

impl<'b, L: QueryLayoutMut + Sync + Send> IndexedParallelIterator for QueryMutParIter<'b, L> {
    fn len(&self) -> usize {
        todo!()
    }

    fn drive<C: rayon::iter::plumbing::Consumer<Self::Item>>(self, consumer: C) -> C::Result {
        todo!()
    }

    fn with_producer<CB: rayon::iter::plumbing::ProducerCallback<Self::Item>>(self, callback: CB) -> CB::Output {
        todo!()
    }
}

impl<'b, L: QueryLayoutMut + Sync + Send> ParallelIterator for QueryMutParIter<'b, L> {
    type Item = L;

    fn drive_unindexed<C>(self, consumer: C) -> C::Result
    where
        C: rayon::iter::plumbing::UnindexedConsumer<Self::Item> {
        bridge(self, consumer)
    }

    fn opt_len(&self) -> Option<usize> {
        Some(self.len())
    }
}

impl<'b, L: QueryLayoutMut + Sync + Send + 'b> IntoParallelIterator for QueryMut<'_, 'b, L> {
    type Iter = QueryMutParIter<'b, L>;
    type Item = L;

    fn into_par_iter(self) -> Self::Iter {
        todo!()
    }
}