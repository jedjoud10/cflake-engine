use crate::{mask, Archetype, Component, Entity, LayoutAccess, Mask};

// Immutable query slice that will be fetched from each archetype
pub trait QueryItemRef: Sized {
    type Slice<'s>: 's;
    type Ptr: 'static + Copy;
    type Owned: 'static;

    // Get the layout access mask for this item
    fn access() -> LayoutAccess;

    // Get a pointer immutable archetypes
    unsafe fn ptr_from_archetype_unchecked(archetype: &Archetype) -> Self::Ptr;

    // Convert the pointer into a slice
    unsafe fn from_raw_parts<'s>(ptr: Self::Ptr, length: usize) -> Self::Slice<'s>;

    // Read from a raw pointer directly
    unsafe fn read_unchecked(ptr: Self::Ptr, index: usize) -> Self;
}

// Mutable query slice that will be fetched from each archetype
pub trait QueryItemMut: Sized {
    type Slice<'s>: 's;
    type Ptr: 'static + Copy;
    type Owned: 'static;

    // Get the layout access mask for this item
    fn access() -> LayoutAccess;

    // Get a pointer from mutable archetypes
    unsafe fn ptr_from_mut_archetype_unchecked(archetype: &mut Archetype) -> Self::Ptr;

    // Convert the pointer into a slice, and read from said slice
    unsafe fn from_raw_parts<'s>(ptr: Self::Ptr, length: usize) -> Self::Slice<'s>;

    // Read from a raw pointer directly
    unsafe fn read_mut_unchecked(ptr: Self::Ptr, index: usize) -> Self;
}

impl<T: Component> QueryItemRef for &T {
    type Slice<'s> = &'s [T];
    type Ptr = *const T;
    type Owned = T;

    fn access() -> LayoutAccess {
        LayoutAccess {
            arch_search: mask::<T>(),
            validation_shared: mask::<T>(),
            validation_unique: Mask::zero(),
        }
    }

    unsafe fn ptr_from_archetype_unchecked(archetype: &Archetype) -> Self::Ptr {
        archetype.components::<T>().unwrap().as_slice().as_ptr() as _
    }

    unsafe fn from_raw_parts<'s>(ptr: Self::Ptr, length: usize) -> Self::Slice<'s> {
        std::slice::from_raw_parts(ptr, length)
    }

    unsafe fn read_unchecked(ptr: Self::Ptr, index: usize) -> Self {
        &*ptr.add(index)
    }
}

impl<T: Component> QueryItemRef for Option<&T> {
    type Slice<'s> = Option<&'s [T]>;
    type Ptr = Option<*const T>;
    type Owned = T;

    fn access() -> LayoutAccess {
        LayoutAccess {
            arch_search: Mask::zero(),
            validation_shared: mask::<T>(),
            validation_unique: Mask::zero(),
        }
    }

    unsafe fn ptr_from_archetype_unchecked(archetype: &Archetype) -> Self::Ptr {
        archetype
            .components::<T>()
            .map(|col| col.as_slice().as_ptr() as _)
    }

    unsafe fn from_raw_parts<'s>(ptr: Self::Ptr, length: usize) -> Self::Slice<'s> {
        ptr.map(|ptr| std::slice::from_raw_parts(ptr, length))
    }

    unsafe fn read_unchecked(ptr: Self::Ptr, index: usize) -> Self {
        ptr.map(|ptr| &*ptr.add(index))
    }
}

impl QueryItemRef for &Entity {
    type Slice<'s> = &'s [Entity];
    type Ptr = *const Entity;
    type Owned = Entity;

    fn access() -> LayoutAccess {
        LayoutAccess {
            arch_search: Mask::zero(),
            validation_shared: Mask::zero(),
            validation_unique: Mask::zero(),
        }
    }

    unsafe fn ptr_from_archetype_unchecked(archetype: &Archetype) -> Self::Ptr {
        archetype.entities().as_ptr()
    }

    unsafe fn from_raw_parts<'s>(ptr: Self::Ptr, length: usize) -> Self::Slice<'s> {
        std::slice::from_raw_parts(ptr, length)
    }

    unsafe fn read_unchecked(ptr: Self::Ptr, index: usize) -> Self {
        &*ptr.add(index)
    }
}

impl<T: Component> QueryItemMut for &T {
    type Slice<'s> = &'s [T];
    type Ptr = *const T;
    type Owned = T;

    fn access() -> LayoutAccess {
        LayoutAccess {
            arch_search: mask::<T>(),
            validation_shared: mask::<T>(),
            validation_unique: Mask::zero(),
        }
    }

    unsafe fn ptr_from_mut_archetype_unchecked(archetype: &mut Archetype) -> Self::Ptr {
        archetype
            .components_mut::<T>()
            .unwrap()
            .as_mut_slice()
            .as_ptr() as _
    }

    unsafe fn from_raw_parts<'s>(ptr: Self::Ptr, length: usize) -> Self::Slice<'s> {
        std::slice::from_raw_parts(ptr, length)
    }

    unsafe fn read_mut_unchecked(ptr: Self::Ptr, index: usize) -> Self {
        &*ptr.add(index)
    }
}

impl<T: Component> QueryItemMut for Option<&T> {
    type Slice<'s> = Option<&'s [T]>;
    type Ptr = Option<*const T>;
    type Owned = T;

    fn access() -> LayoutAccess {
        LayoutAccess {
            arch_search: Mask::zero(),
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

    unsafe fn read_mut_unchecked(ptr: Self::Ptr, index: usize) -> Self {
        ptr.map(|ptr| &*ptr.add(index))
    }
}

impl<T: Component> QueryItemMut for &mut T {
    type Slice<'s> = &'s mut [T];
    type Ptr = *mut T;
    type Owned = T;

    fn access() -> LayoutAccess {
        LayoutAccess {
            arch_search: mask::<T>(),
            validation_shared: Mask::zero(),
            validation_unique: mask::<T>(),
        }
    }

    unsafe fn ptr_from_mut_archetype_unchecked(archetype: &mut Archetype) -> Self::Ptr {
        archetype
            .components_mut::<T>()
            .unwrap()
            .as_mut_slice()
            .as_mut_ptr() as _
    }

    unsafe fn from_raw_parts<'s>(ptr: Self::Ptr, length: usize) -> Self::Slice<'s> {
        std::slice::from_raw_parts_mut(ptr, length)
    }

    unsafe fn read_mut_unchecked(ptr: Self::Ptr, index: usize) -> Self {
        &mut *ptr.add(index)
    }
}

impl<T: Component> QueryItemMut for Option<&mut T> {
    type Slice<'s> = Option<&'s mut [T]>;
    type Ptr = Option<*mut T>;
    type Owned = T;

    fn access() -> LayoutAccess {
        LayoutAccess {
            arch_search: Mask::zero(),
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

    unsafe fn read_mut_unchecked(ptr: Self::Ptr, index: usize) -> Self {
        ptr.map(|ptr| &mut *ptr.add(index))
    }
}

impl QueryItemMut for &Entity {
    type Slice<'s> = &'s [Entity];
    type Ptr = *const Entity;
    type Owned = Entity;

    fn access() -> LayoutAccess {
        LayoutAccess {
            arch_search: Mask::zero(),
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

    unsafe fn read_mut_unchecked(ptr: Self::Ptr, index: usize) -> Self {
        &*ptr.add(index)
    }
}
