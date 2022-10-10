use crate::{Archetype, Component, LayoutAccess, Mask};

// Mutable query layouts that might contain mutable references
// This must take a mutable reference to the current archetype
pub trait MutQueryLayout<'s, 'l>: 'l + Sized {
    type SliceTuple: 's;
    fn is_valid() -> bool;
    fn prepare<'ar: 's>(archetype: &mut Archetype) -> Option<Self::SliceTuple>;
    fn access(archetype_mask: Mask) -> Option<LayoutAccess>;
    fn read(tuple: &mut Self::SliceTuple, index: usize) -> Self;
}

// Immutable query layouts that will never contain any mutable referneces
// This simply takes an immutable reference to the archetype
pub trait RefQueryLayout<'a>: 'a + Sized {
    type PtrTuple: Copy + 'static;
    fn access(archetype_mask: Mask) -> Option<LayoutAccess>;
    fn is_valid() -> bool;
    fn prepare(archetype: &Archetype) -> Option<Self::PtrTuple>;
    unsafe fn read(ptrs: Self::PtrTuple, i: usize) -> Self;
}

// Mutable component items that will be stored within the archetypes
pub trait MutQueryItem<'s: 'l, 'l>: 'l + Sized {
    type Item: 'static;
    type Ptr: 'static + Copy;
    type Slice: 's;
    fn access(archetype_mask: Mask) -> Option<LayoutAccess>;
    fn prepare<'ar: 's>(archetype: &mut Archetype) -> Option<Self::Ptr>;
    unsafe fn into_slice(ptr: Self::Ptr, length: usize) -> Self::Slice; 
    fn read_from_slice(slice: &mut Self::Slice, index: usize) -> Self;
}

// Immutable component items that will be stored within the archetype
pub trait RefQueryItem<'a>: 'a + Sized {
    type Component: 'static + Component;
    type Ptr: 'static + Copy;
    fn access(archetype_mask: Mask) -> Option<LayoutAccess>;
    fn prepare(archetype: &Archetype) -> Option<Self::Ptr>;
    unsafe fn read(ptr: Self::Ptr, i: usize) -> Self;
}
