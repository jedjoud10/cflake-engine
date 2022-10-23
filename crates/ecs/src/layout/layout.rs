use std::alloc::Layout;

use crate::{Archetype, QueryItemRef, LayoutAccess, Mask, QueryItemMut};

// A query layout ref is a combination of multiple immutable query items
// I separated mutable and immutable query for the sake of type safety 
pub trait QueryLayoutRef<'s, 'i> {
    type SliceTuple: 's;
    type OwnedTuple: 'static;

    // Get a combined layout access mask by running a lambda on each layout
    fn reduce(lambda: impl FnMut(LayoutAccess, LayoutAccess) -> LayoutAccess) -> LayoutAccess;

    // Get the query layout slices from an immutable archetype
    unsafe fn slices_from_archetype_unchecked(archetype: &Archetype) -> Self::SliceTuple;

    // Read from the valid slice tuple
    unsafe fn get_unchecked<'a: 'i>(slices: &'a Self::SliceTuple, index: usize) -> Self;
}

// A query layout mut is a combination of multiple mutable/immutable query items
pub trait QueryLayoutMut<'s, 'i>: 'i {
    type SliceTuple: 's;
    type OwnedTuple: 'static;

    // Get a combined layout access mask by running a lambda on each layout
    fn reduce(lambda: impl FnMut(LayoutAccess, LayoutAccess) -> LayoutAccess) -> LayoutAccess;

    // This checks if the layout is valid (no collisions, no ref-mut collisions)
    fn is_valid() -> bool {
        let combined = Self::reduce(|a, b| a | b);
        let a = combined.shared() & combined.unique() == Mask::zero();
        dbg!(a);
        todo!();
        /*
        let mut_items = (0..Self::items()).into_iter().filter(|i| Self::access(*i).unwrap().unique() != Mask::zero()).count() as u64;
        let enabled: u64 = combined.unique().into();
        let b = mut_items == enabled.count_ones() as u64;  
        dbg!(b);
        a && b
        */
    }

    // Check if the query layout contains any mutable items
    fn is_mutable() -> bool {
        let combined = Self::reduce(|a, b| a | b);
        combined.unique() != Mask::zero()
    }

    // Get the query layout slices from a mutable archetype 
    unsafe fn slices_from_mut_archetype_unchecked(archetype: &mut Archetype) -> Self::SliceTuple;

    // Read from the valid slice tuple
    unsafe fn get_mut_unchecked<'a: 'i>(slices: &'a mut Self::SliceTuple, index: usize) -> Self;
}

impl<'s: 'i, 'i, I: QueryItemRef<'s, 'i>> QueryLayoutRef<'s, 'i> for I {
    type SliceTuple = I::Slice;
    type OwnedTuple = I::Owned;

    fn reduce(lambda: impl FnMut(LayoutAccess, LayoutAccess) -> LayoutAccess) -> LayoutAccess {
        std::iter::once(I::access()).into_iter().reduce(lambda).unwrap()
    }

    unsafe fn slices_from_archetype_unchecked(archetype: &Archetype) -> Self::SliceTuple {
        let ptr = I::ptr_from_archetype_unchecked(archetype);
        I::from_raw_parts(ptr, archetype.len())
    }

    unsafe fn get_unchecked<'a: 'i>(slice: &'a Self::SliceTuple, index: usize) -> Self {
        <I as QueryItemRef<'s, 'i>>::get_unchecked(slice, index)
    }
}

impl<'s: 'i, 'i, I: QueryItemMut<'s, 'i> + 'i> QueryLayoutMut<'s, 'i> for I {
    type SliceTuple = I::Slice;
    type OwnedTuple = I::Owned;

    fn reduce(lambda: impl FnMut(LayoutAccess, LayoutAccess) -> LayoutAccess) -> LayoutAccess {
        std::iter::once(I::access()).into_iter().reduce(lambda).unwrap()
    }

    unsafe fn slices_from_mut_archetype_unchecked(archetype: &mut Archetype) -> Self::SliceTuple {
        let ptr = I::ptr_from_mut_archetype_unchecked(archetype);
        I::from_raw_parts(ptr, archetype.len())
    }

    unsafe fn get_mut_unchecked<'a: 'i>(slice: &'a mut Self::SliceTuple, index: usize) -> Self {
        <I as QueryItemMut<'s, 'i>>::get_mut_unchecked(slice, index)
    }
}