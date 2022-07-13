use crate::Component;

// Shared references are implemented for &T only
pub trait SharedReference<'a>: 'a {
    type Inner: Component;

    // Read from the inner value's pointer into &T
    unsafe fn as_ref(ptr: *const Self::Inner) -> Self;
}

impl<'a, T: Component> SharedReference<'a> for &'a T {
    type Inner = T;

    unsafe fn as_ref(ptr: *const Self::Inner) -> Self {
        &*ptr
    }
}

// Generic are either &T references or &mut references. They are "generic" because we hide their inner type
pub trait GenericReference<'a>: 'a {
    type Inner: Component;
    type Ptr: 'static + Copy;
    const MUTABLE: bool;
}

// Generic reference for shared references
impl<'a, T: Component> GenericReference<'a> for &'a T {
    type Inner = T;
    type Ptr = *const T;
    const MUTABLE: bool = false;
}

// Generic reference for unique references
impl<'a, T: Component> GenericReference<'a> for &'a mut T {
    type Inner = T;
    type Ptr = *mut T;
    const MUTABLE: bool = true;
}