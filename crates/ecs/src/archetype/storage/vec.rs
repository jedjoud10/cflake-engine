use crate::component::Component;
use std::{any::Any, ffi::c_void, ptr::NonNull, ops::Range};

// A component storage that is implemented for Vec<UnsafeCell<T>>
pub(crate) trait StorageVec {
    // As any and as any mut
    fn as_any(&self) -> &dyn Any;
    fn as_any_mut(&mut self) -> &mut dyn Any;

    // Vector shit
    fn push(&mut self, component: Box<dyn Any>);
    fn swap_remove(&mut self, bundle: usize);
    fn swap_remove_boxed(&mut self, bundle: usize) -> Box<dyn Any>;
    fn reserve(&mut self, additional: usize);

    // Pointer shit
    fn as_mut_typeless_ptr(&mut self) -> *mut c_void;
    fn get_null_mut_typeless_ptr(&self) -> *mut c_void;

    // Create a new boxed vector (empty)
    fn new_empty_from_self(&self) -> Box<dyn StorageVec>;
}

impl<T: Component> StorageVec for Vec<T> {
    // As any and as any mut
    fn as_any(&self) -> &dyn Any {
        self
    }
    fn as_any_mut(&mut self) -> &mut dyn Any {
        self
    }

    // Push a component into the vector
    // We are 100% sure that the component is of type T
    fn push(&mut self, component: Box<dyn Any>) {
        // Cast the boxed component to T and insert it
        let component = *component.downcast::<T>().unwrap();
        self.push(component);
    }
    // Swap remove an element
    fn swap_remove(&mut self, bundle: usize) {
        self.swap_remove(bundle);
    }
    // Swap remove an element, but box the result
    fn swap_remove_boxed(&mut self, bundle: usize) -> Box<dyn Any> {
        let element = self.swap_remove(bundle);
        Box::new(element)
    }
    fn reserve(&mut self, additional: usize) {
        self.reserve(additional)
    }

    // Pointer shit
    fn as_mut_typeless_ptr(&mut self) -> *mut c_void {
        self.as_mut_ptr() as *mut c_void
    }
    fn get_null_mut_typeless_ptr(&self) -> *mut c_void {
        NonNull::<T>::dangling().as_ptr() as *mut c_void
    }

    // Create a new boxed component storage of an empty vec
    fn new_empty_from_self(&self) -> Box<dyn StorageVec> {
        Box::new(Vec::<T>::new())
    }
}
