use crate::Component;
use std::marker::PhantomData;

// Gets a "&" reference to the component (or entity)
pub struct Read<T: 'static>(&'static T);

// Gets a "&mut" reference to the data
pub struct Write<T: 'static, const SILENT: bool = false>(&'static mut T);

// Trait that will be implmenented for Read<T> and Write<T>
pub trait BorrowedItem<'a> {
    type Component: 'static + Component;
    type Borrowed: 'a;

    // Get the reference from a pointer
    fn read(ptr: *mut Self::Component) -> Self::Borrowed;
}

impl<'a, T: Component> BorrowedItem<'a> for Read<T>
where
    Self: 'a,
{
    type Component = T;
    type Borrowed = &'a T;

    fn read(ptr: *mut Self::Component) -> Self::Borrowed {
        unsafe { &*ptr }
    }
}

impl<'a, T: Component> BorrowedItem<'a> for Write<T>
where
    Self: 'a,
{
    type Component = T;
    type Borrowed = &'a mut T;

    fn read(ptr: *mut Self::Component) -> Self::Borrowed {
        unsafe { &mut *ptr }
    }
}
