use std::{any::{TypeId, type_name}, ptr::NonNull};

use crate::Resource;

// We store the type ID and name in their own struct since the handle might not even be mutable
pub struct ResourceReferenceDesc {
    pub(super) _type: TypeId,
    pub(super) name: &'static str,
    pub(super) mutable: bool,
}

// Generic are either &T references or &mut references. They are "generic" because we hide their inner type
pub(crate) trait ResourceReference<'a>: 'a {
    type Inner: 'static + Sized + Resource;
    type Ptr: 'static + Copy;
    const MUTABLE: bool;

    fn descriptor() -> ResourceReferenceDesc {
        ResourceReferenceDesc {
            _type: TypeId::of::<Self::Inner>(),
            name: type_name::<Self::Inner>(),
            mutable: Self::MUTABLE,
        }
    }

    // The user can always convert mutable pointers (unique -> shared/unique, safe)
    unsafe fn from_non_null(ptr: NonNull<Self::Inner>) -> Self;
}

// Generic reference for shared references
impl<'a, T: Resource> ResourceReference<'a> for &'a T {
    type Inner = T;
    type Ptr = *const T;
    const MUTABLE: bool = false;

    unsafe fn from_non_null(ptr: NonNull<Self::Inner>) -> Self {
        &*ptr.as_ptr()
    }
}

// Generic reference for unique references
impl<'a, T: Resource> ResourceReference<'a> for &'a mut T {
    type Inner = T;
    type Ptr = *mut T;
    const MUTABLE: bool = true;

    unsafe fn from_non_null(ptr: NonNull<Self::Inner>) -> Self {
        &mut *ptr.as_ptr()
    }
}