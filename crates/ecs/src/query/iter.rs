use std::marker::PhantomData;

use crate::{Mask, StorageVecPtr, QueryLayout};

// A custom iterator that will loop over the components of a specific layout

pub struct QueryIter<'a, Layout: QueryLayout<'a>> {
    _phantom: PhantomData<&'a Layout>,
}

impl<'a, Layout: QueryLayout<'a>> Iterator for QueryIter<'a, Layout> {
    type Item = Layout::SafeTuple;

    fn next(&mut self) -> Option<Self::Item> {
        todo!()
    }
}