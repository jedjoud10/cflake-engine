use crate::Component;
use std::{any::Any, ffi::c_void, ptr::NonNull};

// A component storage that is implemented for Vec<T>
pub trait StorageVec {
    // As any and as any mut
    fn as_any(&self) -> &dyn Any;
    fn as_any_mut(&mut self) -> &mut dyn Any;

    // Vector shit
    fn push(&mut self, component: Box<dyn Any>);
    fn swap_remove(&mut self, bundle: usize);
    fn swap_remove_boxed(&mut self, bundle: usize) -> Box<dyn Any>;
    fn reserve(&mut self, additional: usize);

    // Pointer shit
    fn get_storage_ptr(&self) -> NonNull<c_void>;

    // Create a new boxed vector (empty)
    fn clone_unique_storage(&self) -> Box<dyn StorageVec>;
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
    fn get_storage_ptr(&self) -> NonNull<c_void> {
        NonNull::new(self.as_ptr() as *mut c_void).unwrap()
    }
    // Create a new boxed component storage of an empty vec
    fn clone_unique_storage(&self) -> Box<dyn StorageVec> {
        Box::new(Vec::<T>::new())
    }
}
