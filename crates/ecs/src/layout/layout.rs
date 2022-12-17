use crate::{
    Archetype, LayoutAccess, Mask, QueryItemMut, QueryItemRef,
};

// A query layout ref is a combination of multiple immutable query items
// I separated mutable and immutable query for the sake of type safety
pub trait QueryLayoutRef<'s> {
    type SliceTuple: 's;
    type PtrTuple: 'static + Copy;
    type OwnedTuple: 'static;

    // Get a combined layout access mask by running a lambda on each layout
    fn reduce(
        lambda: impl FnMut(LayoutAccess, LayoutAccess) -> LayoutAccess,
    ) -> LayoutAccess;

    // Read ptrs from the archetype, convert to slices, read from pointers
    unsafe fn ptrs_from_archetype_unchecked(
        archetype: &Archetype,
    ) -> Self::PtrTuple;
    unsafe fn from_raw_parts(
        ptrs: Self::PtrTuple,
        length: usize,
    ) -> Self::SliceTuple;
    unsafe fn read_unchecked(
        ptrs: Self::PtrTuple,
        index: usize,
    ) -> Self;
}

// A query layout mut is a combination of multiple mutable/immutable query items
pub trait QueryLayoutMut<'s> {
    type SliceTuple: 's;
    type PtrTuple: 'static + Copy;
    type OwnedTuple: 'static;

    // Get a combined layout access mask by running a lambda on each layout
    fn reduce(
        lambda: impl FnMut(LayoutAccess, LayoutAccess) -> LayoutAccess,
    ) -> LayoutAccess;

    // This checks if the layout is valid (no collisions, no ref-mut collisions)
    fn is_valid() -> bool {
        let combined = Self::reduce(|a, b| a | b);
        let refmut_collisions =
            combined.shared_validation_mask() & combined.unique_validation_mask() != Mask::zero();
            
        let mut mut_collisions = false;
        Self::reduce(|a, b| {
            mut_collisions |= (a.unique_validation_mask() & b.unique_validation_mask()) == b.unique_validation_mask();
            a | b
        });
        !refmut_collisions && !mut_collisions
    }

    // Check if the query layout contains any mutable items
    fn is_mutable() -> bool {
        let combined = Self::reduce(|a, b| a | b);
        combined.unique_validation_mask() != Mask::zero()
    }

    // Read ptrs from the archetype, convert to slices, read from pointers
    unsafe fn ptrs_from_mut_archetype_unchecked(
        archetype: &mut Archetype,
    ) -> Self::PtrTuple;
    unsafe fn from_raw_parts(
        ptrs: Self::PtrTuple,
        length: usize,
    ) -> Self::SliceTuple;
    unsafe fn read_mut_unchecked(
        ptrs: Self::PtrTuple,
        index: usize,
    ) -> Self;
}

impl<'s, I: QueryItemRef<'s> + 's> QueryLayoutRef<'s> for I {
    type OwnedTuple = I::Owned;
    type PtrTuple = I::Ptr;
    type SliceTuple = I::Slice;

    fn reduce(
        lambda: impl FnMut(LayoutAccess, LayoutAccess) -> LayoutAccess,
    ) -> LayoutAccess {
        std::iter::once(I::access())
            .into_iter()
            .reduce(lambda)
            .unwrap()
    }

    unsafe fn ptrs_from_archetype_unchecked(
        archetype: &Archetype,
    ) -> Self::PtrTuple {
        I::ptr_from_archetype_unchecked(archetype)
    }

    unsafe fn from_raw_parts(
        ptrs: Self::PtrTuple,
        length: usize,
    ) -> Self::SliceTuple {
        <I as QueryItemRef<'s>>::from_raw_parts(ptrs, length)
    }

    unsafe fn read_unchecked(
        ptrs: Self::PtrTuple,
        index: usize,
    ) -> Self {
        <I as QueryItemRef<'s>>::read_unchecked(ptrs, index)
    }
}

impl<'s, I: QueryItemMut<'s> + 's> QueryLayoutMut<'s> for I {
    type OwnedTuple = I::Owned;
    type PtrTuple = I::Ptr;
    type SliceTuple = I::Slice;

    fn reduce(
        lambda: impl FnMut(LayoutAccess, LayoutAccess) -> LayoutAccess,
    ) -> LayoutAccess {
        std::iter::once(I::access())
            .into_iter()
            .reduce(lambda)
            .unwrap()
    }

    unsafe fn ptrs_from_mut_archetype_unchecked(
        archetype: &mut Archetype,
    ) -> Self::PtrTuple {
        I::ptr_from_mut_archetype_unchecked(archetype)
    }

    unsafe fn from_raw_parts(
        ptrs: Self::PtrTuple,
        length: usize,
    ) -> Self::SliceTuple {
        <I as QueryItemMut<'s>>::from_raw_parts(ptrs, length)
    }

    unsafe fn read_mut_unchecked(
        ptrs: Self::PtrTuple,
        index: usize,
    ) -> Self {
        <I as QueryItemMut<'s>>::read_mut_unchecked(ptrs, index)
    }
}
