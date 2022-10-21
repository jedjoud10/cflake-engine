use crate::{Archetype, Component, LayoutAccess, Mask, mask, Entity};

// Immutable/mutable query items that will be fetched from each archetype
pub trait QueryItem<'s, 'i>: Sized {
    type Slice: 's;
    type Ptr: 'static + Copy;
    type Owned: 'static;
    const MUTABLE: bool;

    // Get the name of this query item (debugging only)
    fn name() -> &'static str {
        std::any::type_name::<Self::Owned>()
    }

    // Get the layout access mask for this item
    fn access() -> LayoutAccess;
    
    // Get a pointer from mutable and immutable archetypes
    unsafe fn ptr_from_archetype_unchecked(archetype: &Archetype) -> Self::Ptr;
    unsafe fn ptr_from_mut_archetype_unchecked(archetype: &mut Archetype) -> Self::Ptr;

    // Convert the pointer into a slice, and read from said slice
    unsafe fn from_raw_parts(ptr: Self::Ptr, length: usize) -> Self::Slice;
    unsafe fn get_unchecked(slice: Self::Slice, index: usize) -> Self;
}

// Impl of query item for &T components
impl<'s: 'i, 'i, T: Component> QueryItem<'s, 'i> for &'i T {
    type Slice = &'s [T];
    type Ptr = *const T;
    type Owned = T;
    const MUTABLE: bool = false;

    fn access() -> LayoutAccess {
        LayoutAccess::new(mask::<T>(), Mask::zero())
    }
    
    unsafe fn ptr_from_archetype_unchecked(archetype: &Archetype) -> Self::Ptr {
        archetype.table::<T>().unwrap_unchecked().as_slice().as_ptr()
    }

    unsafe fn ptr_from_mut_archetype_unchecked(archetype: &mut Archetype) -> Self::Ptr {
        archetype.table::<T>().unwrap_unchecked().as_slice().as_ptr()
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
    const MUTABLE: bool = true;

    fn access() -> LayoutAccess {
        LayoutAccess::new(Mask::zero(), mask::<T>())
    }
    
    unsafe fn ptr_from_archetype_unchecked(archetype: &Archetype) -> Self::Ptr {
        panic!("This should've never happened wtf did you do jed")
    }

    unsafe fn ptr_from_mut_archetype_unchecked(archetype: &mut Archetype) -> Self::Ptr {
        archetype.table_mut::<T>().unwrap_unchecked().as_mut_slice().as_mut_ptr()
    }

    unsafe fn from_raw_parts(ptr: Self::Ptr, length: usize) -> Self::Slice {
        std::slice::from_raw_parts_mut(ptr, length)
    }

    unsafe fn get_unchecked(slice: Self::Slice, index: usize) -> Self {
        slice.get_unchecked_mut(index)
    }
}

// Impl of query item for Option<&T> components
impl<'s: 'i, 'i, T: Component> QueryItem<'s, 'i> for Option<&'i T> {
    type Slice = Option<&'s [T]>;
    type Ptr = Option<*const T>;
    type Owned = T;
    const MUTABLE: bool = false;

    fn access() -> LayoutAccess {
        LayoutAccess::new(mask::<T>(), Mask::zero())
    }

    unsafe fn ptr_from_archetype_unchecked(archetype: &Archetype) -> Self::Ptr {
        archetype.table::<T>().map(|vec| vec.as_slice().as_ptr())
    }

    unsafe fn ptr_from_mut_archetype_unchecked(archetype: &mut Archetype) -> Self::Ptr {
        archetype.table::<T>().map(|vec| vec.as_slice().as_ptr())
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
    const MUTABLE: bool = false;

    fn access() -> LayoutAccess {
        LayoutAccess::new(Mask::one(), Mask::zero())
    }

    unsafe fn ptr_from_archetype_unchecked(archetype: &Archetype) -> Self::Ptr {
        archetype.entities().as_ptr()
    }

    unsafe fn ptr_from_mut_archetype_unchecked(archetype: &mut Archetype) -> Self::Ptr {
        archetype.entities().as_ptr()
    }

    unsafe fn from_raw_parts(ptr: Self::Ptr, length: usize) -> Self::Slice {
        std::slice::from_raw_parts(ptr, length)
    }

    unsafe fn get_unchecked(slice: Self::Slice, index: usize) -> Self {
        slice.get_unchecked(index)
    }
}