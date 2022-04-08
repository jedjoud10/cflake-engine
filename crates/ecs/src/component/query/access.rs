use std::marker::PhantomData;

use crate::Component;

// (Read, Write) access types. By default, the write access type will also write to the component mutation bitfield
pub trait AccessType {
    type Component;
    fn apply();
}

// Gets a "&" reference to the component
pub struct Read<T: Component>(PhantomData<*const T>);

impl<T: Component> AccessType for Read<T> {
    type Component = T;
    fn apply() {
        
    }
}


// Gets a "&mut" reference to the component
pub struct Write<T: Component, const SILENT: bool = false>(PhantomData<*const T>);

impl<T: Component> AccessType for Write<T> {
    type Component = T;
    fn apply() {
        todo!()
    }
}