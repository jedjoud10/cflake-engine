use itertools::izip;

use crate::{BorrowedItem, QueryCache, QueryError};
// A query layout trait that will be implemented on tuples that contains different types of QueryItems, basically
pub trait QueryLayout<'a> {
    // The tuple that will contain the pointers types of the specific query items
    type PtrTuple: 'static + Copy;

    // The safe tuple that will be given to the user
    type SafeTuple: 'a;

    // Get the ptr tuple chunks from the cache
    fn get_filtered_chunks(cache: &QueryCache) -> Result<Vec<(Self::PtrTuple, usize)>, QueryError>;

    // Read the references from the pointer tuple using a specified offset
    fn read_tuple(tuple: Self::PtrTuple, bundle: usize) -> Self::SafeTuple;
}

impl<'a, A: BorrowedItem<'a>> QueryLayout<'a> for A {
    type PtrTuple = *mut A::Component;
    type SafeTuple = A::Borrowed;

    fn get_filtered_chunks(cache: &QueryCache) -> Result<Vec<(Self::PtrTuple, usize)>, QueryError> {
        let ptrs = cache.get_row::<A>()?;
        let lengths = cache.get_lengths();

        let vec = ptrs
            .iter()
            .zip(lengths.iter())
            .filter_map(|(&ptr, &len)| {
                let a = ptr? as *mut A::Component;
                Some((a, len))
            })
            .collect::<Vec<_>>();
        Ok(vec)
    }

    fn read_tuple(tuple: Self::PtrTuple, bundle: usize) -> Self::SafeTuple {
        A::read(tuple, bundle)
    }
}

impl<'a, A: BorrowedItem<'a>, B: BorrowedItem<'a>> QueryLayout<'a> for (A, B) {
    type PtrTuple = (*mut A::Component, *mut B::Component);
    type SafeTuple = (A::Borrowed, B::Borrowed);

    fn get_filtered_chunks(cache: &QueryCache) -> Result<Vec<(Self::PtrTuple, usize)>, QueryError> {
        let ptrs_a = cache.get_row::<A>()?;
        let ptrs_b = cache.get_row::<B>()?;
        let lengths = cache.get_lengths();

        let vec = izip!(ptrs_a, ptrs_b)
            .zip(lengths.iter())
            .filter_map(|((&a, &b), &len)| {
                let a = a? as *mut A::Component;
                let b = b? as *mut B::Component;
                Some(((a, b), len))
            })
            .collect::<Vec<_>>();
        Ok(vec)
    }

    fn read_tuple(tuple: Self::PtrTuple, bundle: usize) -> Self::SafeTuple {
        (A::read(tuple.0, bundle), B::read(tuple.1, bundle))
    }
}

impl<'a, A: BorrowedItem<'a>, B: BorrowedItem<'a>, C: BorrowedItem<'a>> QueryLayout<'a> for (A, B, C) {
    type PtrTuple = (*mut A::Component, *mut B::Component, *mut C::Component);
    type SafeTuple = (A::Borrowed, B::Borrowed, C::Borrowed);

    fn get_filtered_chunks(cache: &QueryCache) -> Result<Vec<(Self::PtrTuple, usize)>, QueryError> {
        let ptrs_a = cache.get_row::<A>()?;
        let ptrs_b = cache.get_row::<B>()?;
        let ptrs_c = cache.get_row::<C>()?;
        let lengths = cache.get_lengths();

        let vec = izip!(ptrs_a, ptrs_b, ptrs_c)
            .zip(lengths.iter())
            .filter_map(|((&a, &b, &c), &len)| {
                let a = a? as *mut A::Component;
                let b = b? as *mut B::Component;
                let c = c? as *mut C::Component;
                Some(((a, b, c), len))
            })
            .collect::<Vec<_>>();
        Ok(vec)
    }

    fn read_tuple(tuple: Self::PtrTuple, bundle: usize) -> Self::SafeTuple {
        (A::read(tuple.0, bundle), B::read(tuple.1, bundle), C::read(tuple.2, bundle))
    }
}
