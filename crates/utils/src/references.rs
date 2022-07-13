use std::ptr::NonNull;

// Shared references are implemented for &T only
pub trait SharedReference<'a>: 'a {
    type Inner: 'static + Sized;

    // Read from the inner value's pointer into &T
    unsafe fn as_ref(ptr: *const Self::Inner) -> Self;
}

impl<'a, T: 'static + Sized> SharedReference<'a> for &'a T {
    type Inner = T;

    unsafe fn as_ref(ptr: *const Self::Inner) -> Self {
        &*ptr
    }
}

// Unique references are implemented for &mut T only
pub trait UniqueReference<'a>: 'a {
    type Inner: 'static + Sized;

    // Read from the inner value's pointer into &mut T
    unsafe fn as_ref_mut(ptr: *mut Self::Inner) -> Self;
}

impl<'a, T: 'static + Sized> UniqueReference<'a> for &'a mut T {
    type Inner = T;

    unsafe fn as_ref_mut(ptr: *mut Self::Inner) -> Self {
        &mut *ptr
    }
}

// Generic are either &T references or &mut references. They are "generic" because we hide their inner type
pub trait GenericReference<'a>: 'a {
    type Inner: 'static + Sized;
    type Ptr: 'static + Copy;
    const MUTABLE: bool;

    // Read the corresponding pointer to the inner value
    unsafe fn _as(ptr: Self::Ptr) -> Self;

    // The user can always convert mutable pointers (unique -> shared/unique, safe)
    unsafe fn _as_from_mut_ptr(ptr: *mut Self::Inner) -> Self;
}

// Generic reference for shared references
impl<'a, T: 'static + Sized> GenericReference<'a> for &'a T {
    type Inner = T;
    type Ptr = *const T;
    const MUTABLE: bool = false;

    unsafe fn _as(ptr: Self::Ptr) -> Self {
        &*ptr
    }

    unsafe fn _as_from_mut_ptr(ptr: *mut Self::Inner) -> Self {
        &*ptr
    }
}

// Generic reference for unique references
impl<'a, T: 'static + Sized> GenericReference<'a> for &'a mut T {
    type Inner = T;
    type Ptr = *mut T;
    const MUTABLE: bool = true;

    unsafe fn _as(ptr: Self::Ptr) -> Self {
        &mut *ptr
    }

    unsafe fn _as_from_mut_ptr(ptr: *mut Self::Inner) -> Self {
        &mut *ptr
    }
}