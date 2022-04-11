use crate::Component;

// Gets a "&" reference to the component (or entity)
pub struct Read<T: 'static>(&'static T);

// Gets a "&mut" reference to the data
pub struct Write<T: 'static, const SILENT: bool = false>(&'static mut T);

// Trait that will be implmenented for Read<T> and Write<T>
pub trait BorrowedItem<'a> {
    type Component: 'static + Component;
    type Borrowed: 'a;

    // Convert a raw pointer into a borrow (either immutable or mutable) using a specified offset
    fn read(ptr: *mut Self::Component, bundle: usize) -> Self::Borrowed;
}

impl<'a, T: Component> BorrowedItem<'a> for Read<T>
where
    Self: 'a,
{
    type Component = T;
    type Borrowed = &'a T;

    fn read(ptr: *mut Self::Component, bundle: usize) -> Self::Borrowed {
        unsafe { &*ptr.add(bundle) }
    }
}

impl<'a, T: Component> BorrowedItem<'a> for Write<T>
where
    Self: 'a,
{
    type Component = T;
    type Borrowed = &'a mut T;

    fn read(ptr: *mut Self::Component, bundle: usize) -> Self::Borrowed {
        unsafe { &mut *ptr.add(bundle) }
    }
}

// Component filters that only let some components through the query
pub struct Changed<'a, T: BorrowedItem<'a>>(T::Borrowed);

impl<'a, T: BorrowedItem<'a>> BorrowedItem<'a> for Changed<'a, T> {
    type Component = T::Component;
    type Borrowed = T::Borrowed;

    fn read(ptr: *mut Self::Component, bundle: usize) -> Self::Borrowed {
        T::read(ptr, bundle)
    }
}
