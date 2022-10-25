use std::alloc::Layout;

use crate::{Archetype, LayoutAccess, Mask, QuerySliceMut, QuerySliceRef};

// A query layout ref is a combination of multiple immutable query items
// I separated mutable and immutable query for the sake of type safety
pub trait QueryLayoutRef<'i>: 'i {
    type OwnedTuple: 'static;
    type ItemTuple: 'i;

    // Get a combined layout access mask by running a lambda on each layout
    fn reduce(lambda: impl FnMut(LayoutAccess, LayoutAccess) -> LayoutAccess) -> LayoutAccess;

    // Get the query layout slices from an immutable archetype
    unsafe fn slices_from_archetype_unchecked(archetype: &Archetype) -> Self;

    // Read from the valid slice tuple
    unsafe fn get_unchecked<'a: 'i>(slices: &'a Self, index: usize) -> Self::ItemTuple;
}

// A query layout mut is a combination of multiple mutable/immutable query items
pub trait QueryLayoutMut<'i>: 'i {
    type OwnedTuple: 'static;
    type ItemTuple: 'i;

    // Get a combined layout access mask by running a lambda on each layout
    fn reduce(lambda: impl FnMut(LayoutAccess, LayoutAccess) -> LayoutAccess) -> LayoutAccess;

    // This checks if the layout is valid (no collisions, no ref-mut collisions)
    fn is_valid() -> bool {
        let combined = Self::reduce(|a, b| a | b);
        let refmut_collisions = combined.shared() & combined.unique() != Mask::zero();
        let mut mut_collisions = false; 
        Self::reduce(|a, b| {
            mut_collisions |= (a & b) == b;            
            a | b
        });
        
        !refmut_collisions && !mut_collisions
    }

    // Check if the query layout contains any mutable items
    fn is_mutable() -> bool {
        let combined = Self::reduce(|a, b| a | b);
        combined.unique() != Mask::zero()
    }

    // Get the query layout slices from a mutable archetype
    unsafe fn slices_from_mut_archetype_unchecked(archetype: &mut Archetype) -> Self;

    // Read from the valid slice tuple
    unsafe fn get_mut_unchecked<'a: 'i>(slices: &'a mut Self, index: usize) -> Self::ItemTuple;
}

impl<'i, I: QuerySliceRef<'i> + 'i> QueryLayoutRef<'i> for I {
    type OwnedTuple = I::Owned;
    type ItemTuple = I::Item;

    fn reduce(lambda: impl FnMut(LayoutAccess, LayoutAccess) -> LayoutAccess) -> LayoutAccess {
        std::iter::once(I::access())
            .into_iter()
            .reduce(lambda)
            .unwrap()
    }

    unsafe fn slices_from_archetype_unchecked(archetype: &Archetype) -> Self {
        let ptr = I::ptr_from_archetype_unchecked(archetype);
        I::from_raw_parts(ptr, archetype.len())
    }

    unsafe fn get_unchecked<'a: 'i>(slice: &'a Self, index: usize) -> Self::ItemTuple {
        <I as QuerySliceRef<'i>>::get_unchecked(slice, index)
    }
}

impl<'i, I: QuerySliceMut<'i> + 'i> QueryLayoutMut<'i> for I {
    type OwnedTuple = I::Owned;
    type ItemTuple = I::Item;

    fn reduce(lambda: impl FnMut(LayoutAccess, LayoutAccess) -> LayoutAccess) -> LayoutAccess {
        std::iter::once(I::access())
            .into_iter()
            .reduce(lambda)
            .unwrap()
    }

    unsafe fn slices_from_mut_archetype_unchecked(archetype: &mut Archetype) -> Self {
        let ptr = I::ptr_from_mut_archetype_unchecked(archetype);
        I::from_raw_parts(ptr, archetype.len())
    }

    unsafe fn get_mut_unchecked<'a: 'i>(slice: &'a mut Self, index: usize) -> Self::ItemTuple {
        <I as QuerySliceMut<'i>>::get_mut_unchecked(slice, index)
    }
}
