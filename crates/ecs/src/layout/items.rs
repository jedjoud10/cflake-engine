use super::LayoutAccess;
use crate::{
    archetype::Archetype,
    entity::Entity,
    mask::Mask,
    registry::{mask, Component},
};

/// Immutable query slice that will be fetched from each archetype.
pub trait QueryItemRef<'s>: Sized {
    /// Immutable slice of the query item.
    type Slice: 's + Copy;

    /// Get the layout access mask for this item.
    fn access() -> LayoutAccess;

    /// Get a slice from an immutable archetype.
    fn from_archetype(archetype: &'s Archetype) -> Self::Slice;

    /// Read from a the slice directly at a specified index.
    fn read(slice: Self::Slice, index: usize) -> Self;
}

/// Mutable query slice that will be fetched from each archetype.
pub trait QueryItemMut<'s>: Sized {
    /// Immutable slice of the query item.
    type Slice: 's;

    /// Get the layout access mask for this item.
    fn access() -> LayoutAccess;

    /// Get a slice from an mutable archetype.
    fn from_mut_archetype(archetype: &'s mut Archetype) -> Self::Slice;

    /// Read from a the slice directly at a specified index.
    fn read_mut(slice: Self::Slice, index: usize) -> Self;
}

impl<'s, T: Component> QueryItemRef<'s> for &'s T {
    type Slice = &'s [T];

    fn access() -> LayoutAccess {
        LayoutAccess {
            archetype_search: mask::<T>(),
            validation_shared: mask::<T>(),
            validation_unique: Mask::zero(),
        }
    }

    #[inline]
    fn from_archetype(archetype: &'s Archetype) -> Self::Slice {
        archetype.components::<T>().unwrap().as_slice()
    }
    
    #[inline]
    fn read(slice: Self::Slice, index: usize) -> Self {
        &slice[index]
    }
}

/*
impl<T: Component> QueryItemRef for Option<&T> {
    type Slice<'s> = Option<&'s [T]>;

    fn access() -> LayoutAccess {
        LayoutAccess {
            archetype_search: Mask::zero(),
            validation_shared: mask::<T>(),
            validation_unique: Mask::zero(),
        }
    }

    #[inline]
    fn from_archetype<'s>(archetype: &'s Archetype) -> Self::Slice<'s> {
        archetype
            .components::<T>()
            .map(|col| col.as_slice())        
    }
    
    #[inline]
    fn read(slice: Self::Slice<'_>, index: usize) -> Self {
        slice.map(|slice| &slice[index])
    }
}


impl QueryItemRef for &Entity {
    type Slice<'s> = &'s [Entity];

    fn access() -> LayoutAccess {
        LayoutAccess {
            archetype_search: Mask::zero(),
            validation_shared: Mask::zero(),
            validation_unique: Mask::zero(),
        }
    }

    fn from_archetype<'s>(archetype: &'s Archetype) -> Self::Slice<'s> {
        archetype.entities()
    }

    #[inline]
    fn read<'s>(slice: Self::Slice<'s>, index: usize) -> Self where Self: 's {
        &slice[index]
    }
}


impl QueryItemRef for &() {
    type Slice<'s> = &'s [()];

    fn access() -> LayoutAccess {
        LayoutAccess {
            archetype_search: Mask::zero(),
            validation_shared: Mask::zero(),
            validation_unique: Mask::zero(),
        }
    }

    fn from_archetype<'s>(archetype: &'s Archetype) -> Self::Slice<'s> {
        &[]
    }

    #[inline]
    fn read(slice: Self::Slice<'_>, index: usize) -> Self {
        &()
    }
}
*/

impl<'s, T: Component> QueryItemMut<'s> for &'s T {
    type Slice = &'s [T];

    fn access() -> LayoutAccess {
        LayoutAccess {
            archetype_search: mask::<T>(),
            validation_shared: mask::<T>(),
            validation_unique: Mask::zero(),
        }
    }

    #[inline]
    fn from_mut_archetype(archetype: &'s mut Archetype) -> Self::Slice {
        archetype.components_mut::<T>().unwrap().as_mut_slice()
    }
    
    #[inline]
    fn read_mut(slice: Self::Slice, index: usize) -> Self {
        &slice[index]
    }
}

impl<'s, T: Component> QueryItemMut<'s> for &'s mut T {
    type Slice = &'s mut [T];

    fn access() -> LayoutAccess {
        LayoutAccess {
            archetype_search: mask::<T>(),
            validation_shared: Mask::zero(),
            validation_unique: mask::<T>(),
        }
    }

    #[inline]
    fn from_mut_archetype(archetype: &'s mut Archetype) -> Self::Slice {
        archetype.components_mut::<T>().unwrap().as_mut_slice()
    }
    
    #[inline]
    fn read_mut(slice: Self::Slice, index: usize) -> Self {
        &mut slice[index]
    }
}

/*
impl<T: Component> QueryItemMut for Option<&T> {
    type Slice<'s> = Option<&'s [T]>;
    type Ptr = Option<*const T>;

    fn access() -> LayoutAccess {
        LayoutAccess {
            archetype_search: Mask::zero(),
            validation_shared: mask::<T>(),
            validation_unique: Mask::zero(),
        }
    }

    unsafe fn ptr_from_mut_archetype_unchecked(archetype: &mut Archetype) -> Self::Ptr {
        archetype
            .components_mut::<T>()
            .map(|vec| vec.as_slice().as_ptr() as _)
    }

    unsafe fn from_raw_parts<'s>(ptr: Self::Ptr, length: usize) -> Self::Slice<'s> {
        ptr.map(|ptr| std::slice::from_raw_parts(ptr, length))
    }

    #[inline(always)]
    unsafe fn read_mut_unchecked(ptr: Self::Ptr, index: usize) -> Self {
        ptr.map(|ptr| &*ptr.add(index))
    }
}


impl<T: Component> QueryItemMut for Option<&mut T> {
    type Slice<'s> = Option<&'s mut [T]>;
    type Ptr = Option<*mut T>;

    fn access() -> LayoutAccess {
        LayoutAccess {
            archetype_search: Mask::zero(),
            validation_shared: Mask::zero(),
            validation_unique: mask::<T>(),
        }
    }

    unsafe fn ptr_from_mut_archetype_unchecked(archetype: &mut Archetype) -> Self::Ptr {
        archetype
            .components_mut::<T>()
            .map(|vec| vec.as_mut_slice().as_mut_ptr() as _)
    }

    unsafe fn from_raw_parts<'s>(ptr: Self::Ptr, length: usize) -> Self::Slice<'s> {
        ptr.map(|ptr| std::slice::from_raw_parts_mut(ptr, length))
    }

    #[inline(always)]
    unsafe fn read_mut_unchecked(ptr: Self::Ptr, index: usize) -> Self {
        ptr.map(|ptr| &mut *ptr.add(index))
    }
}

impl QueryItemMut for &Entity {
    type Slice<'s> = &'s [Entity];
    type Ptr = *const Entity;

    fn access() -> LayoutAccess {
        LayoutAccess {
            archetype_search: Mask::zero(),
            validation_shared: Mask::zero(),
            validation_unique: Mask::zero(),
        }
    }

    unsafe fn ptr_from_mut_archetype_unchecked(archetype: &mut Archetype) -> Self::Ptr {
        archetype.entities().as_ptr()
    }

    unsafe fn from_raw_parts<'s>(ptr: Self::Ptr, length: usize) -> Self::Slice<'s> {
        std::slice::from_raw_parts(ptr, length)
    }

    #[inline(always)]
    unsafe fn read_mut_unchecked(ptr: Self::Ptr, index: usize) -> Self {
        &*ptr.add(index)
    }
}
*/