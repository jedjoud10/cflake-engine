use crate::{Component, Entity, Archetype, registry, LayoutAccess, Mask};

// Shared references are implemented for &T only
pub trait SharedQueryItemReference<'a>: 'a {
    type Item;

    // Get the base pointer from the archetype
    fn try_fetch_ptr(archetype: &Archetype) -> Option<*const Self::Item>;

    // Read from the inner value's pointer into &T
    unsafe fn as_ref(ptr: *const Self::Item, bundle: isize) -> Self;
}

impl<'a, T: Component> SharedQueryItemReference<'a> for &'a T {
    type Item = T;

    fn try_fetch_ptr(archetype: &Archetype) -> Option<*const Self::Item> {
        let mask = registry::mask::<T>();
        let boxed = archetype.storage().get(&mask)?;
        let ptr = boxed.as_any().downcast_ref::<Vec<T>>().unwrap().as_ptr();
        Some(ptr)
    }

    unsafe fn as_ref(ptr: *const Self::Item, bundle: isize) -> Self {
        &*ptr.offset(bundle)
    }
}

impl<'a> SharedQueryItemReference<'a> for &'a Entity {
    type Item = Entity;

    fn try_fetch_ptr(archetype: &Archetype) -> Option<*const Self::Item> {
        Some(archetype.entities().as_ptr())
    }

    unsafe fn as_ref(ptr: *const Self::Item, bundle: isize) -> Self {
        todo!()
    }
}

// Generic are either &T references or &mut references. They are "generic" because we hide their inner type
pub trait GenericQueryItemReference<'a>: 'a {
    type Item;
    type Ptr: 'static + Copy;
    const MUTABLE: bool;

    // Get the normal component mask and writing mask
    fn read_write_access() -> LayoutAccess;

    // Get the base pointer from the archetype
    fn try_fetch_ptr(archetype: &Archetype) -> Option<*mut Self::Item>;

    // Read from the inner value's pointer into &T
    unsafe fn as_generic_ref(ptr: *const Self::Item, bundle: isize) -> Self;
}

// Generic reference for shared component references
impl<'a, T: Component> GenericQueryItemReference<'a> for &'a T {
    type Item = T;
    type Ptr = *const T;
    const MUTABLE: bool = false;
    
    fn read_write_access() -> LayoutAccess {
        //LayoutAccess(registry::mask::<T>(), Mask::zero())
        todo!()
    }

    fn try_fetch_ptr(archetype: &Archetype) -> Option<*mut Self::Item> {
        todo!()
    }

    unsafe fn as_generic_ref(ptr: *const Self::Item, bundle: isize) -> Self {
        todo!()
    }

}

// Generic reference for unique component references
impl<'a, T: Component> GenericQueryItemReference<'a> for &'a mut T {
    type Item = T;
    type Ptr = *mut T;
    const MUTABLE: bool = true;

    fn try_fetch_ptr(archetype: &Archetype) -> Option<*mut Self::Item> {
        todo!()
    }

    unsafe fn as_generic_ref(ptr: *const Self::Item, bundle: isize) -> Self {
        todo!()
    }

    fn read_write_access() -> LayoutAccess {
        todo!()
    }
}