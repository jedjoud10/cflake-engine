use crate::{registry, Component, ComponentError, ComponentStateSet, Mask, QueryError};
use std::{ffi::c_void, ptr::NonNull};
// Gets a "&" reference to the component
pub struct Read<T: 'static + Component>(&'static T);

// Gets a "&mut" reference to the component
pub struct Write<T: 'static + Component, const SILENT: bool = false>(&'static mut T);

// Trait that will be implmenented for Read<T> and Write<T>
pub trait BorrowedComponent<'a> {
    type Component: 'static + Component;
    type Borrowed: 'a;

    // Offset an unsafe pointer by a bundle offset and read it
    fn offset(ptr: NonNull<Self::Component>, bundle: usize) -> Self::Borrowed;

    // Get the component mask without paining yourself
    fn mask() -> Result<Mask, QueryError> {
        registry::mask::<Self::Component>().map_err(QueryError::ComponentError)
    }
}

impl<'a, T: Component> BorrowedComponent<'a> for Read<T>
where
    Self: 'a,
{
    type Component = T;
    type Borrowed = &'a T;

    fn offset(ptr: NonNull<Self::Component>, bundle: usize) -> Self::Borrowed {
        unsafe { &*ptr.as_ptr().add(bundle) }
    }
}

impl<'a, T: Component, const SILENT: bool> BorrowedComponent<'a> for Write<T, SILENT>
where
    Self: 'a,
{
    type Component = T;
    type Borrowed = &'a mut T;

    fn offset(ptr: NonNull<Self::Component>, bundle: usize) -> Self::Borrowed {
        unsafe { &mut *ptr.as_ptr().add(bundle) }
    }
}
