use crate::{component::Component, registry, ComponentError, StorageVecPtr};
use std::{
    alloc::Layout,
    any::Any,
    cell::UnsafeCell,
    ffi::c_void,
    mem::{align_of, size_of},
};

// A component storage that is implemented for Vec<UnsafeCell<T>>
pub(crate) trait StorageVec {
    // As any and as any mut
    fn as_any(&self) -> &dyn Any;
    fn as_any_mut(&mut self) -> &mut dyn Any;

    // Vector shit
    fn push(&mut self, component: Box<dyn Any>);
    fn swap_remove_bundle(&mut self, bundle: usize);
    fn swap_remove_boxed_bundle(&mut self, bundle: usize) -> Box<dyn Any>;

    // Get a pointer to the underlying data
    fn as_storage_ptr(&mut self) -> Result<StorageVecPtr, ComponentError>;

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
    // Simple swap remove
    fn swap_remove_bundle(&mut self, bundle: usize) {
        self.swap_remove(bundle);
    }
    // Simple swap remove, but box the result
    fn swap_remove_boxed_bundle(&mut self, bundle: usize) -> Box<dyn Any> {
        let element = self.swap_remove(bundle);
        Box::new(element)
    }

    // Le storage vec pointer
    fn as_storage_ptr(&mut self) -> Result<StorageVecPtr, ComponentError> {
        Ok(StorageVecPtr {
            ptr: self.as_mut_ptr() as *mut c_void,
        })
    }

    // Create a new boxed component storage of an empty vec
    fn new_empty_from_self(&self) -> Box<dyn StorageVec> {
        Box::new(Vec::<T>::new())
    }
}
