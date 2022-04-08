use std::marker::PhantomData;
use crate::Component;

// Gets a "&" reference to the component (or entity)
pub struct Read<'a, T>(&'a T);

// Gets a "&mut" reference to the data
pub struct Write<'a, T, const SILENT: bool = false>(&'a mut T);

pub trait ComponentBorrower<'a> {
    type Component;
    
    // The borrwoed component, either &'a T or &'a mut T
    type Borrowed: 'a;
}

impl<'a, T: Component> ComponentBorrower<'a> for Read<'a, T> {
    type Component = T;
    type Borrowed = &'a T;
}

impl<'a, T: Component> ComponentBorrower<'a> for Write<'a, T> {
    type Component = T;
    type Borrowed = &'a mut T;
}