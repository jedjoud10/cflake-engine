use crate::{Component, Entity, Archetype, registry, LayoutAccess, Mask, mask};

// Shared references are implemented for &T only
// Only used for View Queries
pub trait ViewItemReference<'a>: 'a {
    type Item: 'static + Sized;

    // Get the reading mask
    fn read_mask() -> Mask;

    // Get the base pointer from the archetype
    fn try_fetch_ptr(archetype: &Archetype) -> Option<*const Self::Item>;

    // Read from the inner value's pointer into &T
    unsafe fn as_ref(ptr: *const Self::Item, bundle: usize) -> Self;
}

impl<'a, T: Component> ViewItemReference<'a> for &'a T {
    type Item = T;

    fn read_mask() -> Mask {
        mask::<T>()
    }

    fn try_fetch_ptr(archetype: &Archetype) -> Option<*const Self::Item> {
        let mask = registry::mask::<T>();
        let boxed = archetype.storage().get(&mask)?;
        let ptr = boxed.as_any().downcast_ref::<Vec<T>>().unwrap().as_ptr();
        Some(ptr)
    }

    unsafe fn as_ref(ptr: *const Self::Item, bundle: usize) -> Self {
        &*ptr.add(bundle)
    }
}

impl<'a> ViewItemReference<'a> for &'a Entity {
    type Item = Entity;

    fn read_mask() -> Mask {
        Mask::zero()
    }

    fn try_fetch_ptr(archetype: &Archetype) -> Option<*const Self::Item> {
        Some(archetype.entities().as_ptr())
    }

    unsafe fn as_ref(ptr: *const Self::Item, bundle: usize) -> Self {
        &*ptr.add(bundle)
    }
}

// Generic are either &T references or &mut references
// Only used for mutable Queries
pub trait QueryItemReference<'a>: 'a {
    type Item: 'static + Sized;
    type Ptr: 'static + Copy;
    const MUTABLE: bool;

    // Get the normal component mask and writing mask
    fn access() -> LayoutAccess;

    // Get the base pointer from the archetype
    fn try_fetch_ptr(archetype: &mut Archetype) -> Option<Self::Ptr>;

    // Read from the inner value's pointer into &T
    unsafe fn as_self(ptr: Self::Ptr, bundle: usize) -> Self;
}

// Generic reference for shared component references
impl<'a, T: Component> QueryItemReference<'a> for &'a T {
    type Item = T;
    type Ptr = *const T;
    const MUTABLE: bool = false;
    
    fn access() -> LayoutAccess {
        LayoutAccess::new(mask::<T>(), Mask::zero())
    }

    fn try_fetch_ptr(archetype: &mut Archetype) -> Option<Self::Ptr> {
        let boxed = archetype.storage().get(&mask::<T>())?;
        let vec = boxed.as_any().downcast_ref::<Vec<T>>().unwrap();
        Some(vec.as_ptr())
    }

    unsafe fn as_self(ptr: *const Self::Item, bundle: usize) -> Self {
        &*ptr.add(bundle)
    }
}

// Generic reference for unique component references
impl<'a, T: Component> QueryItemReference<'a> for &'a mut T {
    type Item = T;
    type Ptr = *mut T;
    const MUTABLE: bool = true;

    fn access() -> LayoutAccess {
        LayoutAccess::new(Mask::zero(), mask::<T>())
    }

    fn try_fetch_ptr(archetype: &mut Archetype) -> Option<Self::Ptr> {
        let boxed = archetype.storage_mut().get_mut(&mask::<T>())?;
        let vec = boxed.as_any_mut().downcast_mut::<Vec<T>>().unwrap();
        Some(vec.as_mut_ptr())
    }

    unsafe fn as_self(ptr: Self::Ptr, bundle: usize) -> Self {
        &mut *ptr.add(bundle)
    }
}

// Generic reference for shared entity references
impl<'a> QueryItemReference<'a> for &'a Entity {
    type Item = Entity;
    type Ptr = *const Entity;
    const MUTABLE: bool = false;

    fn access() -> LayoutAccess {
        LayoutAccess::none()
    }

    fn try_fetch_ptr(archetype: &mut Archetype) -> Option<Self::Ptr> {
        Some(archetype.entities().as_ptr())
    }

    unsafe fn as_self(ptr: Self::Ptr, bundle: usize) -> Self {
        &*ptr.add(bundle)
    }
}