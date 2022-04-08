use crate::Component;
use std::marker::PhantomData;

// Gets a "&" reference to the component (or entity)
pub struct Read<T: 'static>(&'static T);

// Gets a "&mut" reference to the data
pub struct Write<T: 'static, const SILENT: bool = false>(&'static mut T);

pub trait ComponentBorrower<'a> {
    type Component;

    // The borrwoed component, either &'a T or &'a mut T
    type Borrowed: 'a;
}

impl<'a, T: Component> ComponentBorrower<'a> for Read<T> where Self: 'a {
    type Component = T;
    type Borrowed = &'a T;
}

impl<'a, T: Component> ComponentBorrower<'a> for Write<T> where Self: 'a {
    type Component = T;
    type Borrowed = &'a mut T;
}
