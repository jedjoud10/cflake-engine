use std::{any::Any, cell::UnsafeCell};

use crate::component::Component;

// Implemented for Vectors
pub trait VecComponentStorage: 'static {
    // As any for immutable casting
    fn as_any(&self) -> &dyn Any;
    // As any mut for mutable casting
    fn as_any_mut(&mut self) -> &mut dyn Any;
}

impl<T: Component> VecComponentStorage for Vec<UnsafeCell<T>> {
    fn as_any(&self) -> &dyn Any {
        self
    }

    fn as_any_mut(&mut self) -> &mut dyn Any {
        self
    }
}
