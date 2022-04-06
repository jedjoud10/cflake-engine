use std::marker::PhantomData;

use rayon::iter::ParallelIterator;

use crate::{EcsManager, LayoutQuery};

// Query iterator because we need to assure that the EcsManager does not get mutated while we have a valid query
pub struct QueryIterator<'a, Layout: LayoutQuery<'a> + 'a> {
    pub(super) iterator: std::vec::IntoIter<Layout::Item>,
    pub(super) length: usize,
    pub(super) _phantom: PhantomData<&'a mut EcsManager>,
}

impl<'a, Layout: LayoutQuery<'a> + 'a> Iterator for QueryIterator<'a, Layout> {
    type Item = Layout::Item;

    fn next(&mut self) -> Option<Self::Item> {
        self.iterator.next()
    }

    fn size_hint(&self) -> (usize, Option<usize>) {
        (self.length, Some(self.length))
    }
}

impl<'a, Layout: LayoutQuery<'a> + 'a> ExactSizeIterator for QueryIterator<'a, Layout> {}

// A query iterator that can be used in parallel using rayon
pub struct ParQueryIterator<'a, Layout: LayoutQuery<'a> + 'a> {
    pub(super) iterator: rayon::vec::IntoIter<Layout::Item>,
    pub(super) _phantom: PhantomData<&'a mut ()>,
}

impl<'a, Layout: LayoutQuery<'a> + 'a> ParallelIterator for ParQueryIterator<'a, Layout> {
    type Item = Layout::Item;

    fn drive_unindexed<C>(self, consumer: C) -> C::Result
    where
        C: rayon::iter::plumbing::UnindexedConsumer<Self::Item>,
    {
        self.iterator.drive_unindexed(consumer)
    }
}
