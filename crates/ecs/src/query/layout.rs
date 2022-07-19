use crate::{Archetype, Component, LayoutAccess};

// Mutable query layouts that might contain mutable references
// This must take a mutable reference to the current archetype
pub trait MutQueryLayout<'a>: 'a + Sized {
    type PtrTuple: Copy + 'static;
    fn is_valid() -> bool;
    fn access() -> LayoutAccess;
    fn prepare(archetype: &mut Archetype) -> Option<Self::PtrTuple>;
    unsafe fn read(ptrs: Self::PtrTuple, i: usize) -> Self;
}

// Immutable query layouts that will never contain any mutable referneces
// This simply takes an immutable reference to the archetype
pub trait RefQueryLayout<'a>: 'a + Sized {
    type PtrTuple: Copy + 'static;
    fn access() -> LayoutAccess;
    fn is_valid() -> bool;
    fn prepare(archetype: &Archetype) -> Option<Self::PtrTuple>;
    unsafe fn read(ptrs: Self::PtrTuple, i: usize) -> Self;
}

// Mutable component items that will be stored within the archetypes
pub trait MutQueryItem<'a>: 'a + Sized {
    type Item: 'static + Component;
    fn access() -> LayoutAccess;
    fn prepare(archetype: &mut Archetype) -> Option<*mut Self::Item>;
    unsafe fn read(slice: *mut Self::Item, i: usize) -> Self;
}

// Immutable component items that will be stored within the archetype
pub trait RefQueryItem<'a>: 'a + Sized {
    type Item: 'static + Component;
    fn access() -> LayoutAccess;
    fn prepare(archetype: &Archetype) -> Option<*const Self::Item>;
    unsafe fn read(ptr: *const Self::Item, i: usize) -> Self;
}
