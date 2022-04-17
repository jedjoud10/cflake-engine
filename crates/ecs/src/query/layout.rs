use std::{ffi::c_void, ptr::NonNull, rc::Rc};

use crate::{registry, ComponentStateSet, LayoutAccess, PtrReader, QueryChunk, ComponentStateRow};

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

    // Get the final layout access masks
    fn combined() -> LayoutAccess;

    // Convert the base ptr tuple to the safe borrows using a bundle offset
    fn offset(tuple: Self::PtrTuple, bundle: usize) -> Self::SafeTuple;
}

// Special chunk that allows us to read the SafeTuple from the underlying layout
pub(crate) struct PtrReaderChunk<'a, Layout: QueryLayout<'a>> {
    base: Layout::PtrTuple,
    len: usize,
    states: Rc<ComponentStateSet>,
}

impl<'a, Layout: QueryLayout<'a>> Clone for PtrReaderChunk<'a, Layout> {
    fn clone(&self) -> Self {
        Self {
            base: self.base,
            len: self.len,
            states: self.states.clone(),
        }
    }
}

impl<'a, Layout: QueryLayout<'a>> PtrReaderChunk<'a, Layout> {
    // Create a single reader chunk from a chunk
    pub fn new(chunk: &QueryChunk) -> Self {
        Self {
            base: Layout::ptrs_to_tuple(&chunk.ptrs),
            len: chunk.len,
            states: chunk.states.clone(),
        }
    }

    // Update the components states using the layout access mask. This will return the old states
    pub fn update_states(&self, bundle: usize, access: LayoutAccess) -> Option<ComponentStateRow> {
        self.states.update(bundle, |row| {
            row.update(|_, m| *m = *m | *access.writing())
        })
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
    let ptr = registry::mask::<T::Component>().offset();
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

    fn combined() -> LayoutAccess {
        A::access()
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

    fn combined() -> LayoutAccess {
        A::access() | B::access()
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

    fn combined() -> LayoutAccess {
        A::access() | B::access() | C::access()
    }
}
