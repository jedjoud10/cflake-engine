use crate::{registry, Component, ComponentError, ComponentStateSet, Mask};
use std::ffi::c_void;
// Gets a "&" reference to the component
pub struct Read<T: 'static + Component>(&'static T);

// Gets a "&mut" reference to the component
pub struct Write<T: 'static + Component, const SILENT: bool = false>(&'static mut T);

// Trait that will be implmenented for Read<T> and Write<T>
pub trait BorrowedItem<'a> {
    type Component: 'static + Component;
    type Borrowed: 'a;

    fn offset(ptr: *mut Self::Component, bundle: usize) -> Self::Borrowed;
}

impl<'a, T: Component> BorrowedItem<'a> for Read<T>
where
    Self: 'a,
{
    type Component = T;
    type Borrowed = &'a T;

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

    fn offset(ptr: *mut Self::Component, bundle: usize) -> Self::Borrowed {
        unsafe { &mut *ptr.add(bundle) }
    }
}
