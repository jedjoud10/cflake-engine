use crate::{Archetype, Component, LayoutAccess, Mask, mask, Entity};

// Immutable query items that will be fetched from each archetype
pub trait QueryItemRef<'s, 'i>: Sized {
    type Slice: 's;
    type Ptr: 'static + Copy;
    type Owned: 'static;

    // Get the layout access mask for this item
    fn access() -> LayoutAccess;
    
    // Get a pointer immutable archetypes
    unsafe fn ptr_from_archetype_unchecked(archetype: &Archetype) -> Self::Ptr;

    // Convert the pointer into a slice, and read from said slice
    unsafe fn from_raw_parts(ptr: Self::Ptr, length: usize) -> Self::Slice;
    unsafe fn get_unchecked<'a: 'i>(slice: &'a Self::Slice, index: usize) -> Self;
}

// Mutable query items that will be fetched from each archetype
pub trait QueryItemMut<'s, 'i>: Sized {
    type Slice: 's;
    type Ptr: 'static + Copy;
    type Owned: 'static;

    // Get the layout access mask for this item
    fn access() -> LayoutAccess;
    
    // Get a pointer from mutable archetypes
    unsafe fn ptr_from_mut_archetype_unchecked(archetype: &mut Archetype) -> Self::Ptr;

    // Convert the pointer into a slice, and read from said slice
    unsafe fn from_raw_parts(ptr: Self::Ptr, length: usize) -> Self::Slice;
    unsafe fn get_mut_unchecked<'a: 'i>(slice: &'a mut Self::Slice, index: usize) -> Self;
}

impl<'s: 'i, 'i, T: Component> QueryItemRef<'s, 'i> for &'i T {
    type Slice = &'s [T];
    type Ptr = *const T;
    type Owned = T;

    fn access() -> LayoutAccess {
        LayoutAccess::new(mask::<T>(), Mask::zero())
    }
    
    unsafe fn ptr_from_archetype_unchecked(archetype: &Archetype) -> Self::Ptr {
        archetype.table::<T>().unwrap_unchecked().as_slice().as_ptr()
    }

    unsafe fn from_raw_parts(ptr: Self::Ptr, length: usize) -> Self::Slice {
        std::slice::from_raw_parts(ptr, length)
    }

    unsafe fn get_unchecked<'a: 'i>(slice: &'a Self::Slice, index: usize) -> Self {
        slice.get_unchecked(index)
    }
}


impl<'s: 'i, 'i, T: Component> QueryItemRef<'s, 'i> for Option<&'i T> {
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

    unsafe fn get_unchecked<'a: 'i>(slice: &'a Self::Slice, index: usize) -> Self {
        slice.map(|slice| slice.get_unchecked(index))
    }
}

impl<'s: 'i, 'i> QueryItemRef<'s, 'i> for &'i Entity {
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

    unsafe fn get_unchecked<'a: 'i>(slice: &'a Self::Slice, index: usize) -> Self {
        slice.get_unchecked(index)
    }
}

impl<'s: 'i, 'i, T: Component> QueryItemMut<'s, 'i> for &'i T {
    type Slice = &'s [T];
    type Ptr = *const T;
    type Owned = T;

    fn access() -> LayoutAccess {
        LayoutAccess::new(mask::<T>(), Mask::zero())
    }
    
    unsafe fn ptr_from_mut_archetype_unchecked(archetype: &mut Archetype) -> Self::Ptr {
        archetype.table::<T>().unwrap_unchecked().as_slice().as_ptr()
    }

    unsafe fn from_raw_parts(ptr: Self::Ptr, length: usize) -> Self::Slice {
        std::slice::from_raw_parts(ptr, length)
    }

    unsafe fn get_mut_unchecked<'a: 'i>(slice: &'a mut Self::Slice, index: usize) -> Self {
        slice.get_unchecked(index)
    }
}

impl<'s: 'i, 'i, T: Component> QueryItemMut<'s, 'i> for Option<&'i T> {
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

    unsafe fn get_mut_unchecked<'a: 'i>(slice: &'a mut Self::Slice, index: usize) -> Self {
        slice.map(|slice| slice.get_unchecked(index))
    }
}

impl<'s: 'i, 'i, T: Component> QueryItemMut<'s, 'i> for &'i mut T {
    type Slice = &'s mut [T];
    type Ptr = *mut T;
    type Owned = T;

    fn access() -> LayoutAccess {
        LayoutAccess::new(Mask::zero(), mask::<T>())
    }
    
    unsafe fn ptr_from_mut_archetype_unchecked(archetype: &mut Archetype) -> Self::Ptr {
        archetype.table_mut::<T>().unwrap_unchecked().as_mut_slice().as_mut_ptr()
    }

    unsafe fn from_raw_parts(ptr: Self::Ptr, length: usize) -> Self::Slice {
        std::slice::from_raw_parts_mut(ptr, length)
    }

    unsafe fn get_mut_unchecked<'a: 'i>(slice: &'a mut Self::Slice, index: usize) -> Self {
        slice.get_unchecked_mut(index)
    }
}

impl<'s: 'i, 'i, T: Component> QueryItemMut<'s, 'i> for Option<&'i mut T> {
    type Slice = Option<&'s mut [T]>;
    type Ptr = Option<*mut T>;
    type Owned = T;

    fn access() -> LayoutAccess {
        LayoutAccess::new(Mask::zero(), mask::<T>())
    }

    unsafe fn ptr_from_mut_archetype_unchecked(archetype: &mut Archetype) -> Self::Ptr {
        archetype.table_mut::<T>().map(|vec| vec.as_mut_slice().as_mut_ptr())
    }

    unsafe fn from_raw_parts(ptr: Self::Ptr, length: usize) -> Self::Slice {
        ptr.map(|ptr| std::slice::from_raw_parts_mut(ptr, length))
    }

    unsafe fn get_mut_unchecked<'a: 'i>(slice: &'a mut Self::Slice, index: usize) -> Self {
        slice.as_mut().map(|slice| <[T]>::get_unchecked_mut(slice, index))
    }
}

impl<'s: 'i, 'i> QueryItemMut<'s, 'i> for &'i Entity {
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

    unsafe fn get_mut_unchecked<'a: 'i>(slice: &'a mut Self::Slice, index: usize) -> Self {
        slice.get_unchecked(index)
    }
}