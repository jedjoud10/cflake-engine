use crate::component::Component;
use std::{any::Any, cell::UnsafeCell};

// A component storage that is implemented for Vec<UnsafeCell<T>>
pub trait ComponentStorage {
    // As any and as any mut
    fn as_any(&self) -> &dyn Any;
    fn as_any_mut(&mut self) -> &mut dyn Any;
    // Push a component into the vector
    fn push(&mut self, component: Box<dyn Any>);
    // Remove a component without moving the whole vector
    fn swap_remove_bundle(&mut self, bundle: usize);
    // Remove a component, but boxes the result
    fn swap_remove_boxed_bundle(&mut self, bundle: usize) -> Box<dyn Any>;
    // Create a new empty boxed component storage vec using self
    fn new_empty_from_self(&self) -> Box<dyn ComponentStorage>;
}

impl<T: Component> ComponentStorage for Vec<UnsafeCell<T>> {
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
        self.push(UnsafeCell::new(component));
    }
    // Simple swap remove
    fn swap_remove_bundle(&mut self, bundle: usize) {
        self.swap_remove(bundle);
    }
    // Simple swap remove, but box the result
    fn swap_remove_boxed_bundle(&mut self, bundle: usize) -> Box<dyn Any> {
        let element = self.swap_remove(bundle).into_inner();
        Box::new(element)
    }

    // Create a new boxed component storage of an empty vec
    fn new_empty_from_self(&self) -> Box<dyn ComponentStorage> {
        Box::new(Vec::<UnsafeCell<T>>::new())
    }
}
