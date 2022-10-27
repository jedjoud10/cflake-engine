use crate::{mask, Archetype, Component, Entity, LayoutAccess, Mask};

// Immutable query slice that will be fetched from each archetype
pub trait QueryItemRef<'s>: Sized {
    type Slice: 's;
    type Ptr: 'static + Copy;
    type Owned: 'static;

    // Get the layout access mask for this item
    fn access() -> LayoutAccess;

    // Get a pointer immutable archetypes
    unsafe fn ptr_from_archetype_unchecked(archetype: &Archetype) -> Self::Ptr;

    // Convert the pointer into a slice
    unsafe fn from_raw_parts(ptr: Self::Ptr, length: usize) -> Self::Slice;

    // Read from a raw pointer directly
    unsafe fn read_unchecked(ptr: Self::Ptr, index: usize) -> Self;
}

// Mutable query slice that will be fetched from each archetype
pub trait QueryItemMut<'s>: Sized {
    type Slice: 's;
    type Ptr: 'static + Copy;
    type Owned: 'static;

    // Get the layout access mask for this item
    fn access() -> LayoutAccess;

    // Get a pointer from mutable archetypes
    unsafe fn ptr_from_mut_archetype_unchecked(archetype: &mut Archetype) -> Self::Ptr;

    // Convert the pointer into a slice, and read from said slice
    unsafe fn from_raw_parts(ptr: Self::Ptr, length: usize) -> Self::Slice;

    // Read from a raw pointer directly
    unsafe fn read_mut_unchecked(ptr: Self::Ptr, index: usize) -> Self;
}

impl<'s, T: Component> QueryItemRef<'s> for &T {
    type Slice = &'s [T];
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

    unsafe fn from_raw_parts(ptr: Self::Ptr, length: usize) -> Self::Slice {
        std::slice::from_raw_parts(ptr, length)
    }

    unsafe fn read_unchecked(ptr: Self::Ptr, index: usize) -> Self {
        &*ptr.add(index)
    }
}

impl<'s, T: Component> QueryItemRef<'s> for Option<&T> {
    type Slice = Option<&'s [T]>;
    type Ptr = Option<*const T>;
    type Owned = T;

    fn access() -> LayoutAccess {
        LayoutAccess::new(mask::<T>(), Mask::zero())
    }

    unsafe fn ptr_from_archetype_unchecked(archetype: &Archetype) -> Self::Ptr {
        archetype.table::<T>().map(|vec| vec.as_slice().as_ptr())
    }

    unsafe fn from_raw_parts(ptr: Self::Ptr, length: usize) -> Self::Slice {
        ptr.map(|ptr| std::slice::from_raw_parts(ptr, length))
    }

    unsafe fn read_unchecked(ptr: Self::Ptr, index: usize) -> Self {
        ptr.map(|ptr| &*ptr.add(index))
    }
}

impl<'s> QueryItemRef<'s> for &Entity {
    type Slice = &'s [Entity];
    type Ptr = *const Entity;
    type Owned = Entity;

    fn access() -> LayoutAccess {
        LayoutAccess::new(Mask::one(), Mask::zero())
    }

    unsafe fn ptr_from_archetype_unchecked(archetype: &Archetype) -> Self::Ptr {
        archetype.entities().as_ptr()
    }

    unsafe fn from_raw_parts(ptr: Self::Ptr, length: usize) -> Self::Slice {
        std::slice::from_raw_parts(ptr, length)
    }

    unsafe fn read_unchecked(ptr: Self::Ptr, index: usize) -> Self {
        &*ptr.add(index)
    }
}

impl<'s, T: Component> QueryItemMut<'s> for &T {
    type Slice = &'s [T];
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

    unsafe fn from_raw_parts(ptr: Self::Ptr, length: usize) -> Self::Slice {
        std::slice::from_raw_parts(ptr, length)
    }

    unsafe fn read_mut_unchecked(ptr: Self::Ptr, index: usize) -> Self {
        &*ptr.add(index)
    }
}

impl<'s, T: Component> QueryItemMut<'s> for Option<&T> {
    type Slice = Option<&'s [T]>;
    type Ptr = Option<*const T>;
    type Owned = T;

    fn access() -> LayoutAccess {
        LayoutAccess::new(mask::<T>(), Mask::zero())
    }

    unsafe fn ptr_from_mut_archetype_unchecked(archetype: &mut Archetype) -> Self::Ptr {
        archetype.table::<T>().map(|vec| vec.as_slice().as_ptr())
    }

    unsafe fn from_raw_parts(ptr: Self::Ptr, length: usize) -> Self::Slice {
        ptr.map(|ptr| std::slice::from_raw_parts(ptr, length))
    }

    unsafe fn read_mut_unchecked(ptr: Self::Ptr, index: usize) -> Self {
        ptr.map(|ptr| &*ptr.add(index))
    }
}

impl<'s, T: Component> QueryItemMut<'s> for &mut T {
    type Slice = &'s mut [T];
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

    unsafe fn from_raw_parts(ptr: Self::Ptr, length: usize) -> Self::Slice {
        std::slice::from_raw_parts_mut(ptr, length)
    }

    unsafe fn read_mut_unchecked(ptr: Self::Ptr, index: usize) -> Self {
        &mut *ptr.add(index)
    }
}

impl<'s, T: Component> QueryItemMut<'s> for Option<&mut T> {
    type Slice = Option<&'s mut [T]>;
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

    unsafe fn from_raw_parts(ptr: Self::Ptr, length: usize) -> Self::Slice {
        ptr.map(|ptr| std::slice::from_raw_parts_mut(ptr, length))
    }

    unsafe fn read_mut_unchecked(ptr: Self::Ptr, index: usize) -> Self {
        ptr.map(|ptr| &mut *ptr.add(index))
    }
}

impl<'s> QueryItemMut<'s> for &Entity {
    type Slice = &'s [Entity];
    type Ptr = *const Entity;
    type Owned = Entity;

    fn access() -> LayoutAccess {
        LayoutAccess::new(Mask::one(), Mask::zero())
    }

    unsafe fn ptr_from_mut_archetype_unchecked(archetype: &mut Archetype) -> Self::Ptr {
        archetype.entities().as_ptr()
    }

    unsafe fn from_raw_parts(ptr: Self::Ptr, length: usize) -> Self::Slice {
        std::slice::from_raw_parts(ptr, length)
    }

    unsafe fn read_mut_unchecked(ptr: Self::Ptr, index: usize) -> Self {
        &*ptr.add(index)
    }
}
