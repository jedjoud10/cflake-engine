use crate::Component;
use std::{any::Any};

// A component storage that is implemented for Vec<T>
// This type makes a lot of assumption about the parameters 
// that are given to it, so it is only used internally
pub(crate) trait ComponentTable {
    // Runtime dynamic conversions
    fn as_any(&self) -> &dyn Any;
    fn as_any_mut(&mut self) -> &mut dyn Any;

    // Remove a component from the storage, and move the last element into it's place instead
    fn swap_remove(&mut self, index: usize);

    // Remove a component from the storage, and insert the return value into another component storage
    // This assumes that "other" is of the same type as Self
    fn swap_remove_move(&mut self, index: usize, other: &mut dyn ComponentTable);

    // Reserve some allocation space for the storage
    fn reserve(&mut self, additional: usize);

    // This will create an empty ComponentTable vector using another one
    fn default(&self) -> Box<dyn ComponentTable>;
}

impl<T: Component> ComponentTable for Vec<T> {
    // Convert to immutably Any trait object
    fn as_any(&self) -> &dyn Any {
        self
    }

    // Convert to mutable Any trait object
    fn as_any_mut(&mut self) -> &mut dyn Any {
        self
    }

    // Calls to Vec::swap_remove but typeless
    fn swap_remove(&mut self, index: usize) {
        self.swap_remove(index);
    }

    // Calls to Vec::swap_remove, and inserts the result into another storage
    fn swap_remove_move(&mut self, index: usize, other: &mut dyn ComponentTable) {
        let removed = self.swap_remove(index);
        let other = other.as_any_mut().downcast_mut::<Self>().unwrap();
        other.push(removed);
    }

    fn reserve(&mut self, additional: usize) {
        self.reserve(additional);
    }

    fn default(&self) -> Box<dyn ComponentTable> {
        Box::new(Vec::<T>::new())
    }
}
