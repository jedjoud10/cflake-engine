use std::{rc::Rc, ptr::NonNull, ffi::c_void};

use crate::{registry, BorrowedComponent, ComponentError, ComponentStateSet, Mask, QueryCache, QueryError, QueryChunk};
use itertools::izip;

// A query layout trait that will be implemented on tuples that contains different types of QueryItems, basically

// This burns my eyeballs

pub trait QueryLayout<'a> where Self: Sized {
    // Types
    type PtrTuple: 'static + Copy;
    type SafeTuple: 'a;


    // Get the pointer tuple from raw pointers
    fn ptrs_to_tuple(ptrs: &[Option<NonNull<c_void>>; 64]) -> Option<Self::PtrTuple>; 

    // Get the combined masks
    fn mask() -> Result<Mask, QueryError>;
    
    // Convert the base ptr tuple to the safe borrows using a bundle offset 
    fn offset(tuple: Self::PtrTuple, bundle: usize) -> Self::SafeTuple;
}

// Special chunk that allows us to read the SafeTuple from the underlying layout 
pub struct PtrReaderChunk<'a, Layout: QueryLayout<'a>> {
    base: Layout::PtrTuple,
    len: usize,
    states: Rc<ComponentStateSet>,
}

impl<'a, Layout: QueryLayout<'a>> Clone for PtrReaderChunk<'a, Layout> {
    fn clone(&self) -> Self {
        Self { base: self.base.clone(), len: self.len.clone(), states: self.states.clone() }
    }
}

impl<'a, Layout: QueryLayout<'a>> PtrReaderChunk<'a, Layout> {
    // Create a vector of multiple reader chunks from cache
    pub fn query(cache: &QueryCache) -> Result<Vec<Self>, QueryError> {
        // Cache the layout mask for later use
        let mask = Layout::mask()?;

        // Get all the cache chunks
        let chunks = cache.view();

        // Create the readers
        let readers = chunks
            .iter()
            .filter_map(|chunk| {
                // Check if the chunk's mask validates the layout's mask
                (chunk.mask & mask == mask).then(|| {
                    Self {
                        base: Layout::ptrs_to_tuple(&chunk.ptrs).unwrap(),
                        len: chunk.len,
                        states: chunk.states.clone(),
                    }
                })
            })
            .collect::<Vec<_>>();

        Ok(readers)
    }
    // Get the safe borrowing tuple from the chunk
    pub fn get(&self, bundle: usize) -> Option<Layout::SafeTuple> {
        // Handle invalid index
        if bundle == self.len {
            return None;
        }       
        
        // Le offset
        Some(Layout::offset(self.base, bundle))
    }
}

impl<'a, A: BorrowedComponent<'a>> QueryLayout<'a> for A {
    type PtrTuple = NonNull<A::Component>;
    type SafeTuple = A::Borrowed;

    /*
    fn get_filtered_chunks(cache: &QueryCache) -> Result<Vec<Self::PtrTuple>, QueryError> {
        let ptrs = cache.view::<A>()?;

        let vec = ptrs.iter().filter_map(|&ptr| Some(ptr?.as_ptr() as *mut A::Component)).collect::<Vec<_>>();
        Ok(vec)
    }
    */


    fn mask() -> Result<Mask, QueryError> {
        A::mask()
    }

    fn offset(tuple: Self::PtrTuple, bundle: usize) -> Self::SafeTuple {
        A::offset(tuple, bundle)
    }

    fn ptrs_to_tuple(ptrs: &[Option<NonNull<c_void>>; 64]) -> Option<Self::PtrTuple> {
        // Get mask indices
        let a = A::mask().ok()?.offset();
        let ptr = ptrs[a]?.cast::<A::Component>();
        Some(ptr)
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