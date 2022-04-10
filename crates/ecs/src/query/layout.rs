use smallvec::SmallVec;

use crate::{registry, Archetype, BorrowedItem, ComponentError, Mask, QueryCache, StorageVecPtr, QueryError};
// A query layout trait that will be implemented on tuples that contains different types of QueryItems, basically
pub trait QueryLayout<'a> {
    // The tuple that will contain the pointers types of the specific query items
    type PtrTuple: 'static + Copy;

    // The safe tuple that will be given to the user
    type SafeTuple: 'a;

    // Get the ptr tuple chunks from the cache
    fn get_filtered_chunks(cache: &QueryCache) -> Result<Vec<(Self::PtrTuple, usize)>, QueryError>;

    // Get the combined mask of the query layout.
    fn layout_mask() -> Result<Mask, ComponentError>;

    // Read the references from the pointer tuple using a specified offset
    fn read(tuple: Self::PtrTuple, bundle: usize) -> Self::SafeTuple;
}

impl<'a, A: BorrowedItem<'a>> QueryLayout<'a> for A {
    type PtrTuple = *mut A::Component;
    type SafeTuple = A::Borrowed;

    fn get_filtered_chunks(cache: &QueryCache) -> Result<Vec<(Self::PtrTuple, usize)>, QueryError> {
        let collumns = cache.get_row::<A::Component>()?;
        let lengths = cache.get_lengths();
        let vec =  collumns
            .iter()
            .zip(lengths.iter())
            .filter_map(|(&ptr, &len)| {
                let a = ptr? as *mut A::Component;
                Some((a, len))
            }).collect::<Vec<_>>();        
        Ok(vec)
    }

    fn layout_mask() -> Result<Mask, ComponentError> {
        registry::mask::<A::Component>()
    }

    fn read(tuple: Self::PtrTuple, bundle: usize) -> Self::SafeTuple {
        <A as BorrowedItem>::read(tuple, bundle)
    }
}

impl<'a, A: BorrowedItem<'a>, B: BorrowedItem<'a>> QueryLayout<'a> for (A, B) {
    type PtrTuple = (*mut A::Component, *mut B::Component);
    type SafeTuple = (A::Borrowed, B::Borrowed);

    fn get_filtered_chunks(cache: &QueryCache) -> Result<Vec<(Self::PtrTuple, usize)>, QueryError> {
        let collumns_a = cache.get_row::<A::Component>()?;
        let collumns_b = cache.get_row::<B::Component>()?;
        let lengths = cache.get_lengths();
        let vec = collumns_a.iter().zip(collumns_b.iter()).zip(lengths.iter()).filter_map(|((&a, &b), &len)| {
            let a = a? as *mut A::Component;
            let b = b? as *mut B::Component;
            Some(((a, b), len))
        }).collect::<Vec<_>>();
        Ok(vec)
    }

    fn layout_mask() -> Result<Mask, ComponentError> {
        Ok(registry::mask::<A::Component>()? | registry::mask::<A::Component>()?)
    }

    fn read(tuple: Self::PtrTuple, bundle: usize) -> Self::SafeTuple {
        (<A as BorrowedItem>::read(tuple.0, bundle), <B as BorrowedItem>::read(tuple.1, bundle))
    }
}
