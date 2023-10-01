use std::ops::DerefMut;

use crate::{archetype::Archetype, mask::Mask};

use super::{LayoutAccess, QueryItemMut, QueryItemRef};

/// A query layout ref is a combination of multiple immutable query items.
/// I separated mutable and immutable query for the sake of type safety.
pub trait QueryLayoutRef<'s> {
    /// Immutable tuple containing multiple slices of the query items.
    type SliceTuple: 's + Copy;

    /// Get a combined layout access mask by running a lambda on each layout.
    fn reduce(lambda: impl FnMut(LayoutAccess, LayoutAccess) -> LayoutAccess) -> LayoutAccess;

    /// Get the slices from an immutable archetype.
    fn from_archetype(archetype: &'s Archetype) -> Self::SliceTuple;

    /// Read from the slice directly at a specified index
    fn read(slice: Self::SliceTuple, index: usize) -> Self;
}

/// A query layout mut is a combination of multiple mutable/immutable query items.
pub trait QueryLayoutMut<'s> {
    /// Immutable tuple containing multiple slices of the query items.
    type SliceTuple: 's;

    /// Get a combined layout access mask by running a lambda on each layout.
    fn reduce(lambda: impl FnMut(LayoutAccess, LayoutAccess) -> LayoutAccess) -> LayoutAccess;

    /// This checks if the layout is valid (no collisions, no ref-mut collisions)
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

    /// Check if the query layout contains any mutable items.
    fn is_mutable() -> bool {
        let combined = Self::reduce(|a, b| a | b);
        combined.unique() != Mask::zero()
    }

    /// Get the slices from an mutable archetype.
    fn from_mut_archetype(archetype: &'s mut Archetype) -> Self::SliceTuple;

    /// Read from the slice tuple directly at a specified index, consuming the slice tuple
    fn read_mut(slice: Self::SliceTuple, index: usize) -> Self;
}

impl<'s, I: QueryItemRef<'s>> QueryLayoutRef<'s> for I {
    type SliceTuple = I::Slice;

    fn reduce(_lambda: impl FnMut(LayoutAccess, LayoutAccess) -> LayoutAccess) -> LayoutAccess {
        I::access()
    }

    #[inline]
    fn from_archetype(archetype: &'s Archetype) -> Self::SliceTuple {
        <I as QueryItemRef>::from_archetype(archetype)
    }

    #[inline]
    fn read(slice: Self::SliceTuple, index: usize) -> Self {
        <I as QueryItemRef>::read(slice, index)
    }
}

impl<'s, I: QueryItemMut<'s>> QueryLayoutMut<'s> for I {
    type SliceTuple = I::Slice;

    fn reduce(_lambda: impl FnMut(LayoutAccess, LayoutAccess) -> LayoutAccess) -> LayoutAccess {
        I::access()
    }

    #[inline]
    fn from_mut_archetype(archetype: &'s mut Archetype) -> Self::SliceTuple {
        <I as QueryItemMut>::from_mut_archetype(archetype)
    }

    #[inline]
    fn read_mut(slice: Self::SliceTuple, index: usize) -> Self {
        <I as QueryItemMut>::read_mut(slice, index)
    }
}
