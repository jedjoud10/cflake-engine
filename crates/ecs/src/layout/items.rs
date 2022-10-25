use crate::{mask, Archetype, Component, Entity, LayoutAccess, Mask};

// Immutable query slice that will be fetched from each archetype
pub trait QuerySliceRef<'i>: Sized {
    type Item: 'i;
    type Ptr: 'static + Copy;
    type Owned: 'static;

    // Get the layout access mask for this item
    fn access() -> LayoutAccess;

    // Get a pointer immutable archetypes
    unsafe fn ptr_from_archetype_unchecked(archetype: &Archetype) -> Self::Ptr;

    // Convert the pointer into a slice, and read from said slice
    unsafe fn from_raw_parts(ptr: Self::Ptr, length: usize) -> Self;
    unsafe fn get_unchecked<'a: 'i>(slice: &'a Self, index: usize) -> Self::Item;
}

// Mutable query slice that will be fetched from each archetype
pub trait QuerySliceMut<'i>: Sized {
    type Item: 'i;
    type Ptr: 'static + Copy;
    type Owned: 'static;

    // Get the layout access mask for this item
    fn access() -> LayoutAccess;

    // Get a pointer from mutable archetypes
    unsafe fn ptr_from_mut_archetype_unchecked(archetype: &mut Archetype) -> Self::Ptr;

    // Convert the pointer into a slice, and read from said slice
    unsafe fn from_raw_parts(ptr: Self::Ptr, length: usize) -> Self;
    unsafe fn get_mut_unchecked<'a: 'i>(slice: &'a mut Self, index: usize) -> Self::Item;
}

impl<'i, T: Component> QuerySliceRef<'i> for &[T] {
    type Item = &'i T;
    type Ptr = *const T;
    type Owned = T;

    fn access() -> LayoutAccess {
        LayoutAccess::new(mask::<T>(), Mask::zero())
    }

    unsafe fn ptr_from_archetype_unchecked(archetype: &Archetype) -> Self::Ptr {
        archetype
            .table::<T>()
            .unwrap_unchecked()
            .as_slice()
            .as_ptr()
    }

    unsafe fn from_raw_parts(ptr: Self::Ptr, length: usize) -> Self {
        std::slice::from_raw_parts(ptr, length)
    }

    unsafe fn get_unchecked<'a: 'i>(slice: &'a Self, index: usize) -> Self::Item {
        slice.get_unchecked(index)
    }
}

impl<'i, T: Component> QuerySliceRef<'i> for Option<&'i [T]> {
    type Item = Option<&'i T>;
    type Ptr = Option<*const T>;
    type Owned = T;

    fn access() -> LayoutAccess {
        LayoutAccess::new(mask::<T>(), Mask::zero())
    }

    unsafe fn ptr_from_archetype_unchecked(archetype: &Archetype) -> Self::Ptr {
        archetype.table::<T>().map(|vec| vec.as_slice().as_ptr())
    }

    unsafe fn from_raw_parts(ptr: Self::Ptr, length: usize) -> Self {
        ptr.map(|ptr| std::slice::from_raw_parts(ptr, length))
    }

    unsafe fn get_unchecked<'a: 'i>(slice: &'a Self, index: usize) -> Self::Item {
        slice.map(|slice| slice.get_unchecked(index))
    }
}

impl<'i> QuerySliceRef<'i> for &[Entity] {
    type Item = &'i Entity;
    type Ptr = *const Entity;
    type Owned = Entity;

    fn access() -> LayoutAccess {
        LayoutAccess::new(Mask::one(), Mask::zero())
    }

    unsafe fn ptr_from_archetype_unchecked(archetype: &Archetype) -> Self::Ptr {
        archetype.entities().as_ptr()
    }

    unsafe fn from_raw_parts(ptr: Self::Ptr, length: usize) -> Self {
        std::slice::from_raw_parts(ptr, length)
    }

    unsafe fn get_unchecked<'a: 'i>(slice: &'a Self, index: usize) -> Self::Item {
        slice.get_unchecked(index)
    }
}

impl<'i, T: Component> QuerySliceMut<'i> for &[T] {
    type Item = &'i T;
    type Ptr = *const T;
    type Owned = T;

    fn access() -> LayoutAccess {
        LayoutAccess::new(mask::<T>(), Mask::zero())
    }

    unsafe fn ptr_from_mut_archetype_unchecked(archetype: &mut Archetype) -> Self::Ptr {
        archetype
            .table::<T>()
            .unwrap_unchecked()
            .as_slice()
            .as_ptr()
    }

    unsafe fn from_raw_parts(ptr: Self::Ptr, length: usize) -> Self {
        std::slice::from_raw_parts(ptr, length)
    }

    unsafe fn get_mut_unchecked<'a: 'i>(slice: &'a mut Self, index: usize) -> Self::Item {
        slice.get_unchecked(index)
    }
}

impl<'i, T: Component> QuerySliceMut<'i> for Option<&[T]> {
    type Item = Option<&'i T>;
    type Ptr = Option<*const T>;
    type Owned = T;

    fn access() -> LayoutAccess {
        LayoutAccess::new(mask::<T>(), Mask::zero())
    }

    unsafe fn ptr_from_mut_archetype_unchecked(archetype: &mut Archetype) -> Self::Ptr {
        archetype.table::<T>().map(|vec| vec.as_slice().as_ptr())
    }

    unsafe fn from_raw_parts(ptr: Self::Ptr, length: usize) -> Self {
        ptr.map(|ptr| std::slice::from_raw_parts(ptr, length))
    }

    unsafe fn get_mut_unchecked<'a: 'i>(slice: &'a mut Self, index: usize) -> Self::Item {
        slice.map(|slice| slice.get_unchecked(index))
    }
}

impl<'i, T: Component> QuerySliceMut<'i> for &mut [T] {
    type Item = &'i mut T;
    type Ptr = *mut T;
    type Owned = T;

    fn access() -> LayoutAccess {
        LayoutAccess::new(Mask::zero(), mask::<T>())
    }

    unsafe fn ptr_from_mut_archetype_unchecked(archetype: &mut Archetype) -> Self::Ptr {
        archetype
            .table_mut::<T>()
            .unwrap_unchecked()
            .as_mut_slice()
            .as_mut_ptr()
    }

    unsafe fn from_raw_parts(ptr: Self::Ptr, length: usize) -> Self {
        std::slice::from_raw_parts_mut(ptr, length)
    }

    unsafe fn get_mut_unchecked<'a: 'i>(slice: &'a mut Self, index: usize) -> Self::Item {
        slice.get_unchecked_mut(index)
    }
}

impl<'i, T: Component> QuerySliceMut<'i> for Option<&mut [T]> {
    type Item = Option<&'i mut T>;
    type Ptr = Option<*mut T>;
    type Owned = T;

    fn access() -> LayoutAccess {
        LayoutAccess::new(Mask::zero(), mask::<T>())
    }

    unsafe fn ptr_from_mut_archetype_unchecked(archetype: &mut Archetype) -> Self::Ptr {
        archetype
            .table_mut::<T>()
            .map(|vec| vec.as_mut_slice().as_mut_ptr())
    }

    unsafe fn from_raw_parts(ptr: Self::Ptr, length: usize) -> Self {
        ptr.map(|ptr| std::slice::from_raw_parts_mut(ptr, length))
    }

    unsafe fn get_mut_unchecked<'a: 'i>(slice: &'a mut Self, index: usize) -> Self::Item {
        slice
            .as_mut()
            .map(|slice| <[T]>::get_unchecked_mut(slice, index))
    }
}

impl<'i> QuerySliceMut<'i> for &[Entity] {
    type Item = &'i Entity;
    type Ptr = *const Entity;
    type Owned = Entity;

    fn access() -> LayoutAccess {
        LayoutAccess::new(Mask::one(), Mask::zero())
    }

    unsafe fn ptr_from_mut_archetype_unchecked(archetype: &mut Archetype) -> Self::Ptr {
        archetype.entities().as_ptr()
    }

    unsafe fn from_raw_parts(ptr: Self::Ptr, length: usize) -> Self {
        std::slice::from_raw_parts(ptr, length)
    }

    unsafe fn get_mut_unchecked<'a: 'i>(slice: &'a mut Self, index: usize) -> Self::Item {
        slice.get_unchecked(index)
    }
}
