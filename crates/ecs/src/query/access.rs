use crate::{registry, Component, ComponentError, ComponentStateSet, Mask};
use std::ffi::c_void;

// Access masks. Depict how we will access a speficic component
pub struct AccessMask {
    // Write<T> accesses
    pub writing: Mask,

    // Read<T> accesses
    pub reading: Mask,
}

impl AccessMask {
    // Create a new access mask for reading only
    pub fn reading<T: Component>() -> Result<Self, ComponentError> {
        Ok(Self {
            writing: Default::default(),
            reading: registry::mask::<T>()?,
        })
    }
    // Create a new access mask for reading and writing
    pub fn writing<T: Component>() -> Result<Self, ComponentError> {
        Ok(Self {
            writing: registry::mask::<T>()?,
            reading: registry::mask::<T>()?,
        })
    }
}

impl std::ops::BitOr for AccessMask {
    type Output = Self;

    fn bitor(self, rhs: Self) -> Self::Output {
        Self {
            writing: self.writing | rhs.writing,
            reading: self.reading | rhs.reading,
        }
    }
}

// Gets a "&" reference to the component
pub struct Read<T: 'static + Component>(&'static T);

// Gets a "&mut" reference to the component
pub struct Write<T: 'static + Component, const SILENT: bool = false>(&'static mut T);

// Trait that will be implmenented for Read<T> and Write<T>
pub trait BorrowedItem<'a> {
    type Component: 'static + Component;
    type Borrowed: 'a;

    fn access_mask() -> Result<AccessMask, ComponentError>;
    fn offset(ptr: *mut Self::Component, bundle: usize) -> Self::Borrowed;
}

impl<'a, T: Component> BorrowedItem<'a> for Read<T>
where
    Self: 'a,
{
    type Component = T;
    type Borrowed = &'a T;

    fn access_mask() -> Result<AccessMask, ComponentError> {
        AccessMask::reading::<Self::Component>()
    }

    fn offset(ptr: *mut Self::Component, bundle: usize) -> Self::Borrowed {
        unsafe { &*ptr.add(bundle) }
    }
}

impl<'a, T: Component, const SILENT: bool> BorrowedItem<'a> for Write<T, SILENT>
where
    Self: 'a,
{
    type Component = T;
    type Borrowed = &'a mut T;

    fn access_mask() -> Result<AccessMask, ComponentError> {
        if SILENT {
            AccessMask::reading::<Self::Component>()
        } else {
            AccessMask::writing::<Self::Component>()
        }
    }

    fn offset(ptr: *mut Self::Component, bundle: usize) -> Self::Borrowed {
        unsafe { &mut *ptr.add(bundle) }
    }
}
