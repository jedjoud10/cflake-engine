use crate::{FromWorld, World};
pub use resources_derive::Resource;
use std::{any::Any, marker::PhantomData, ptr::NonNull};

// A resource is a global data type that will be stored within the world for the duration of the program
// Resources can be shared amongst events, thus allowing us to share data between ECS systems
pub trait Resource: 'static {
    // Runtime conversions
    fn as_any(&self) -> &dyn Any;
    fn as_any_mut(&mut self) -> &mut dyn Any;

    // Called the moment we create reading/writing guards
    fn write_guard_init(&mut self) {}
    fn read_guard_init(&self) {}
}
