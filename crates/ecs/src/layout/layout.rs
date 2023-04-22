use crate::{Archetype, LayoutAccess, Mask, QueryItemMut, QueryItemRef};

// A query layout ref is a combination of multiple immutable query items
// I separated mutable and immutable query for the sake of type safety
pub trait QueryLayoutRef {
    type SliceTuple<'s>: 's;
    type PtrTuple: 'static + Copy;
    type OwnedTuple: 'static;

    // Get a combined layout access mask by running a lambda on each layout
    fn reduce(lambda: impl FnMut(LayoutAccess, LayoutAccess) -> LayoutAccess) -> LayoutAccess;

    // Read ptrs from the archetype, convert to slices, read from pointers
    unsafe fn ptrs_from_archetype_unchecked(archetype: &Archetype) -> Self::PtrTuple;
    unsafe fn from_raw_parts<'s>(ptrs: Self::PtrTuple, length: usize) -> Self::SliceTuple<'s>;
    unsafe fn read_unchecked(ptrs: Self::PtrTuple, index: usize) -> Self;
}

// A query layout mut is a combination of multiple mutable/immutable query items
pub trait QueryLayoutMut {
    type SliceTuple<'s>: 's;
    type PtrTuple: 'static + Copy;
    type OwnedTuple: 'static;

    // Get a combined layout access mask by running a lambda on each layout
    fn reduce(lambda: impl FnMut(LayoutAccess, LayoutAccess) -> LayoutAccess) -> LayoutAccess;

    // This checks if the layout is valid (no collisions, no ref-mut collisions)
    fn is_valid() -> bool {
        // Check for ref-mut collisions
        let combined = Self::reduce(|a, b| a | b);
        let refmut_collisions = combined.shared() & combined.unique() != Mask::zero();

        // Check for mut collisions between the masks
        let mut mut_collisions = false;
        Self::reduce(|acc, b| {
            mut_collisions |= (acc.unique() & b.unique()) == b.unique() && (!b.unique().is_zero());
            acc | b
        });

        !refmut_collisions && !mut_collisions
    }

    // Check if the query layout contains any mutable items
    fn is_mutable() -> bool {
        let combined = Self::reduce(|a, b| a | b);
        combined.unique() != Mask::zero()
    }

    // Read ptrs from the archetype, convert to slices, read from pointers
    unsafe fn ptrs_from_mut_archetype_unchecked(archetype: &mut Archetype) -> Self::PtrTuple;
    unsafe fn from_raw_parts<'s>(ptrs: Self::PtrTuple, length: usize) -> Self::SliceTuple<'s>;
    unsafe fn read_mut_unchecked(ptrs: Self::PtrTuple, index: usize) -> Self;
}

impl<I: QueryItemRef> QueryLayoutRef for I {
    type OwnedTuple = I::Owned;
    type PtrTuple = I::Ptr;
    type SliceTuple<'s> = I::Slice<'s>;

    fn reduce(lambda: impl FnMut(LayoutAccess, LayoutAccess) -> LayoutAccess) -> LayoutAccess {
        std::iter::once(I::access())
            .into_iter()
            .reduce(lambda)
            .unwrap()
    }

    unsafe fn ptrs_from_archetype_unchecked(archetype: &Archetype) -> Self::PtrTuple {
        I::ptr_from_archetype_unchecked(archetype)
    }

    unsafe fn from_raw_parts<'s>(ptrs: Self::PtrTuple, length: usize) -> Self::SliceTuple<'s> {
        <I as QueryItemRef>::from_raw_parts(ptrs, length)
    }

    unsafe fn read_unchecked(ptrs: Self::PtrTuple, index: usize) -> Self {
        <I as QueryItemRef>::read_unchecked(ptrs, index)
    }
}

impl<I: QueryItemMut> QueryLayoutMut for I {
    type OwnedTuple = I::Owned;
    type PtrTuple = I::Ptr;
    type SliceTuple<'s> = I::Slice<'s>;

    fn reduce(lambda: impl FnMut(LayoutAccess, LayoutAccess) -> LayoutAccess) -> LayoutAccess {
        std::iter::once(I::access())
            .into_iter()
            .reduce(lambda)
            .unwrap()
    }

    unsafe fn ptrs_from_mut_archetype_unchecked(archetype: &mut Archetype) -> Self::PtrTuple {
        I::ptr_from_mut_archetype_unchecked(archetype)
    }

    unsafe fn from_raw_parts<'s>(ptrs: Self::PtrTuple, length: usize) -> Self::SliceTuple<'s> {
        <I as QueryItemMut>::from_raw_parts(ptrs, length)
    }

    unsafe fn read_mut_unchecked(ptrs: Self::PtrTuple, index: usize) -> Self {
        <I as QueryItemMut>::read_mut_unchecked(ptrs, index)
    }
}
