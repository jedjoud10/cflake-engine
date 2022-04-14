use std::{ffi::c_void, ptr::NonNull, rc::Rc};

use crate::{registry, PtrReader, ComponentError, ComponentStateSet, Mask, QueryCache, QueryChunk, QueryError};
use itertools::izip;

// A query layout trait that will be implemented on tuples that contains different types of QueryItems, basically

// This burns my eyeballs

pub trait QueryLayout<'a>
where
    Self: Sized,
{
    // Types
    type PtrTuple: 'static + Copy;
    type SafeTuple: 'a;

    // Get the pointer tuple from raw pointers
    fn ptrs_to_tuple(ptrs: &[Option<NonNull<c_void>>; 64]) -> Self::PtrTuple;

    // Get the normal combined component mask AND writing mask
    fn get_masks() -> Result<(Mask, Mask), QueryError>;

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
        Self {
            base: self.base.clone(),
            len: self.len.clone(),
            states: self.states.clone(),
        }
    }
}

impl<'a, Layout: QueryLayout<'a>> PtrReaderChunk<'a, Layout> {
    // Create a vector of multiple reader chunks from cache
    pub fn query(cache: &QueryCache,) -> Result<(Vec<Self>, Mask), QueryError> {
        // Cache the layout mask for later use
        let (mask, writing_mask) = Layout::get_masks()?;

        // Get all the cache chunks
        let chunks = cache.view();

        // Create the readers
        let readers = chunks
            .iter()
            .filter_map(|chunk| {
                // Check if the chunk's mask validates the layout's mask
                (chunk.mask & mask == mask).then(|| Self {
                    base: Layout::ptrs_to_tuple(&chunk.ptrs),
                    len: chunk.len,
                    states: chunk.states.clone(),
                })
            })
            .collect::<Vec<_>>();

        Ok((readers, writing_mask))
    }
    // Set the component mutation state for a specific mask
    pub fn set_states_for_mask(&self, bundle: usize, mask: Mask) -> Option<()> {
        self.states.set(bundle, mask)
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

// Helper function to get the base pointer of a specific borrowed component
fn get_then_cast<'a, T: PtrReader<'a>>(ptrs: &[Option<NonNull<c_void>>; 64]) -> NonNull<T::Component> {
    let ptr = T::mask().unwrap().0.offset();
    ptrs[ptr].unwrap().cast()
}

impl<'a, A: PtrReader<'a>> QueryLayout<'a> for A {
    type PtrTuple = NonNull<A::Component>;
    type SafeTuple = A::Borrowed;

    fn ptrs_to_tuple(ptrs: &[Option<NonNull<c_void>>; 64]) -> Self::PtrTuple {
        get_then_cast::<A>(ptrs)
    }

    fn offset(tuple: Self::PtrTuple, bundle: usize) -> Self::SafeTuple {
        A::offset(tuple, bundle)
    }

    fn get_masks() -> Result<(Mask, Mask), QueryError> {
        A::mask()
    }
}

impl<'a, A: PtrReader<'a>, B: PtrReader<'a>> QueryLayout<'a> for (A, B) {
    type PtrTuple = (NonNull<A::Component>, NonNull<B::Component>);
    type SafeTuple = (A::Borrowed, B::Borrowed);

    fn ptrs_to_tuple(ptrs: &[Option<NonNull<c_void>>; 64]) -> Self::PtrTuple {
        (get_then_cast::<A>(ptrs), get_then_cast::<B>(ptrs))
    }

    fn offset(tuple: Self::PtrTuple, bundle: usize) -> Self::SafeTuple {
        (A::offset(tuple.0, bundle), B::offset(tuple.1, bundle))
    }

    fn get_masks() -> Result<(Mask, Mask), QueryError> {
        let (a, aw) = A::mask()?;
        let (b, bw) = B::mask()?;
        Ok((a | b, aw | bw))
    }
}

impl<'a, A: PtrReader<'a>, B: PtrReader<'a>, C: PtrReader<'a>> QueryLayout<'a> for (A, B, C) {
    type PtrTuple = (NonNull<A::Component>, NonNull<B::Component>, NonNull<C::Component>);
    type SafeTuple = (A::Borrowed, B::Borrowed, C::Borrowed);

    fn ptrs_to_tuple(ptrs: &[Option<NonNull<c_void>>; 64]) -> Self::PtrTuple {
        (get_then_cast::<A>(ptrs), get_then_cast::<B>(ptrs), get_then_cast::<C>(ptrs))
    }

    fn offset(tuple: Self::PtrTuple, bundle: usize) -> Self::SafeTuple {
        (A::offset(tuple.0, bundle), B::offset(tuple.1, bundle), C::offset(tuple.2, bundle))
    }

    fn get_masks() -> Result<(Mask, Mask), QueryError> {
        let (a, aw) = A::mask()?;
        let (b, bw) = B::mask()?;
        let (c, cw) = C::mask()?;
        Ok((a | b | c, aw | bw | cw))
    }
}