use crate::{Events, Layout, ResourceError, World};
use ahash::AHashMap;
use std::any::{Any, TypeId};

// A resource is some shared data that will be accessed by multiple systems
// This resource cannot be removed from the systems. To be able to remove resources, we must implement the Removable trait as well
pub trait Resource: 'static {
    // Conversions to dynamic any
    fn as_any(&self) -> &dyn Any;
    fn as_any_mut(&mut self) -> &mut dyn Any;

    // This will try to get a pointer to the unique resource that is stored within the world
    // This is stored within the trait to allow the user to write 
}

// This trait hints that the underlying resource can be removed from the world
// Resources are removable by default, though we can opt out of that by using the #[Persistent] attribute
pub trait Removable {

}