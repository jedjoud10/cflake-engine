use std::rc::Rc;

use crate::{registry, AccessMask, BorrowedItem, ComponentError, ComponentStateSet, Mask, QueryCache, QueryError, QueryChunk};
use itertools::izip;

// A query layout trait that will be implemented on tuples that contains different types of QueryItems, basically

// This burns my eyeballs

pub trait QueryLayout<'a> where Self: Sized {
    // Tuple types
    type PtrTuple: 'static + Copy;
    type SafeTuple: 'a;

    // Get the chunks specifically for this layout
    fn chunks(cache: &QueryCache) -> Result<Vec<QueryChunk<'a, Self>>, QueryError>;
    fn layout_access_mask() -> Result<AccessMask, ComponentError>;
    fn read_tuple(tuple: Self::PtrTuple, bundle: usize) -> Self::SafeTuple;
}

// Layout chunk that contains the pointers by themselves
pub struct QueryLayoutChunk<'a, Layout: QueryLayout<'a>> {
    base: Layout::PtrTuple,
    len: usize,
    states: Rc<ComponentStateSet>,
}

impl<'a, Layout: QueryLayout<'a>> QueryLayoutChunk<'a, Layout> {
    // Create a new layout chunk from the query cache
}

impl<'a, A: BorrowedItem<'a>> QueryLayout<'a> for A {
    type PtrTuple = *mut A::Component;
    type SafeTuple = A::Borrowed;

    /*
    fn get_filtered_chunks(cache: &QueryCache) -> Result<Vec<Self::PtrTuple>, QueryError> {
        let ptrs = cache.view::<A>()?;

        let vec = ptrs.iter().filter_map(|&ptr| Some(ptr?.as_ptr() as *mut A::Component)).collect::<Vec<_>>();
        Ok(vec)
    }
    */

    fn read_tuple(tuple: Self::PtrTuple, bundle: usize) -> Self::SafeTuple {
        A::offset(tuple, bundle)
    }

    fn layout_access_mask() -> Result<AccessMask, ComponentError> {
        A::access_mask()
    }

    fn chunks(cache: &QueryCache) -> Result<Vec<CacheChunk<'a, Self>>, QueryError> {
        let ptrs = cache.view::<A>()?;
        /*
        let len = &cache.lengths;
        let states = &cache.states;

        let vec = zipped.filter_map(|(ptr, len, states)| {
            let ptr = ptr.as_ref()?;
            let chunk = CacheChunk::<'a, Self> {
                length: *len,
                states: *states,
                ptrs: ptr.as_ptr() as _,
            };
            Some(ptr)
        });
        
        */
        todo!()
    }
}
/*
impl<'a, A: BorrowedItem<'a>, B: BorrowedItem<'a>> QueryLayout<'a> for (A, B) {
    type PtrTuple = (*mut A::Component, *mut B::Component);
    type SafeTuple = (A::Borrowed, B::Borrowed);

    fn get_filtered_chunks(cache: &QueryCache) -> Result<Vec<Self::PtrTuple>, QueryError> {
        let ptrs_a = cache.view::<A>()?;
        let ptrs_b = cache.view::<B>()?;

        let vec = izip!(ptrs_a, ptrs_b)
            .filter_map(|(&a, &b)| {
                let a = a?.as_ptr() as *mut A::Component;
                let b = b?.as_ptr() as *mut B::Component;
                Some((a, b))
            })
            .collect::<Vec<_>>();
        Ok(vec)
    }

    fn read_tuple(tuple: Self::PtrTuple, bundle: usize) -> Self::SafeTuple {
        (A::offset(tuple.0, bundle), B::offset(tuple.1, bundle))
    }

    fn layout_access_mask() -> Result<AccessMask, ComponentError> {
        Ok(A::access_mask()? | B::access_mask()?)
    }
}

impl<'a, A: BorrowedItem<'a>, B: BorrowedItem<'a>, C: BorrowedItem<'a>> QueryLayout<'a> for (A, B, C) {
    type PtrTuple = (*mut A::Component, *mut B::Component, *mut C::Component);
    type SafeTuple = (A::Borrowed, B::Borrowed, C::Borrowed);

    fn get_filtered_chunks(cache: &QueryCache) -> Result<Vec<Self::PtrTuple>, QueryError> {
        let ptrs_a = cache.view::<A>()?;
        let ptrs_b = cache.view::<B>()?;
        let ptrs_c = cache.view::<C>()?;

        let vec = izip!(ptrs_a, ptrs_b, ptrs_c)
            .filter_map(|(&a, &b, &c)| {
                let a = a?.as_ptr() as *mut A::Component;
                let b = b?.as_ptr() as *mut B::Component;
                let c = c?.as_ptr() as *mut C::Component;
                Some((a, b, c))
            })
            .collect::<Vec<_>>();
        Ok(vec)
    }

    fn read_tuple(tuple: Self::PtrTuple, bundle: usize) -> Self::SafeTuple {
        (A::offset(tuple.0, bundle), B::offset(tuple.1, bundle), C::offset(tuple.2, bundle))
    }

    fn layout_access_mask() -> Result<AccessMask, ComponentError> {
        Ok(A::access_mask()? | B::access_mask()? | C::access_mask()?)
    }
}
*/