use crate::{Archetype, Component, LayoutAccess, Mask, mask, Entity};

// Immutable/mutable query items that will be fetched from each archetype
pub trait QueryItem<'s, 'i>: Sized {
    type Slice: 's;
    type Ptr: 'static + Copy;
    type Owned: 'static;

    fn access() -> LayoutAccess;
    fn get_slice<'a: 's>(archetype: &'a Archetype) -> Self::Slice;
    fn as_ptr(slice: Self::Slice) -> Self::Ptr;
    unsafe fn get_unchecked(slice: Self::Slice, index: usize) -> Self;
    unsafe fn from_raw_parts(ptr: Self::Ptr, length: usize) -> Self::Slice;
}

// Impl of query item for &T components
impl<'s: 'i, 'i, T: Component> QueryItem<'s, 'i> for &'i T {
    type Slice = &'s [T];
    type Ptr = *const T;
    type Owned = T;

    fn access() -> LayoutAccess {
        LayoutAccess::new(mask::<T>(), Mask::zero())
    }

    fn get_slice<'a: 's>(archetype: &'a Archetype) -> Self::Slice {
        archetype.table::<T>().unwrap().as_slice()
    }

    fn as_ptr(slice: Self::Slice) -> Self::Ptr {
        slice.as_ptr()
    }

    unsafe fn from_raw_parts(ptr: Self::Ptr, length: usize) -> Self::Slice {
        std::slice::from_raw_parts(ptr, length)
    }

    unsafe fn get_unchecked(slice: Self::Slice, index: usize) -> Self {
        slice.get_unchecked(index)
    }
}


// Impl of query item for &mut T components
impl<'s: 'i, 'i, T: Component> QueryItem<'s, 'i> for &'i mut T {
    type Slice = &'s mut [T];
    type Ptr = *mut T;
    type Owned = T;

    fn access() -> LayoutAccess {
        LayoutAccess::new(Mask::zero(), mask::<T>())
    }

    fn get_slice<'a: 's>(archetype: &'a Archetype) -> Self::Slice {
        archetype.table_mut::<T>().unwrap().as_slice()
    }

    fn as_ptr(slice: Self::Slice) -> Self::Ptr {
        slice.as_ptr()
    }

    unsafe fn from_raw_parts(ptr: Self::Ptr, length: usize) -> Self::Slice {
        std::slice::from_raw_parts(ptr, length)
    }

    unsafe fn get_unchecked(slice: Self::Slice, index: usize) -> Self {
        slice.get_unchecked(index)
    }
}

// Impl of query item for Option<&T> components
impl<'s: 'i, 'i, T: Component> QueryItem<'s, 'i> for Option<&'i T> {
    type Slice = Option<&'s [T]>;
    type Ptr = Option<*const T>;
    type Owned = T;

    fn access() -> LayoutAccess {
        LayoutAccess::new(mask::<T>(), Mask::zero())
    }

    fn get_slice<'a: 's>(archetype: &'a Archetype) -> Self::Slice {
        archetype.table::<T>().map(|vec| vec.as_slice())
    }

    fn as_ptr(slice: Self::Slice) -> Self::Ptr {
        slice.map(|slice| slice.as_ptr())
    }

    unsafe fn from_raw_parts(ptr: Self::Ptr, length: usize) -> Self::Slice {
        ptr.map(|ptr| std::slice::from_raw_parts(ptr, length))
    }

    unsafe fn get_unchecked(slice: Self::Slice, index: usize) -> Self {
        slice.map(|slice| slice.get_unchecked(index))
    }
}

// Impl of query item for &Entities
impl<'s: 'i, 'i> QueryItem<'s, 'i> for &'i Entity {
    type Slice = &'s [Entity];
    type Ptr = *const Entity;
    type Owned = Entity;

    fn access() -> LayoutAccess {
        LayoutAccess::none()
    }

    fn get_slice<'a: 's>(archetype: &'a Archetype) -> Self::Slice {
        archetype.entities()
    }

    fn as_ptr(slice: Self::Slice) -> Self::Ptr {
        slice.as_ptr()
    }

    unsafe fn from_raw_parts(ptr: Self::Ptr, length: usize) -> Self::Slice {
        std::slice::from_raw_parts(ptr, length)
    }

    unsafe fn get_unchecked(slice: Self::Slice, index: usize) -> Self {
        slice.get_unchecked(index)
    }
}