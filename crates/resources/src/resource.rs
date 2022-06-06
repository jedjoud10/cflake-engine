use std::{
    any::{Any, TypeId},
    cell::RefCell,
};

use ahash::AHashMap;

// A resource set simply contains multiple unique resources
pub type ResourceSet = AHashMap<TypeId, Box<dyn Resource>>;

// A resource is some shared data that will be accessed by multiple systems
pub trait Resource: 'static {
    fn as_any(&self) -> &dyn Any;
    fn as_any_mut(&mut self) -> &mut dyn Any;
}
