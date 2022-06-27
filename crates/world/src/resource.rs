use crate::{Events, Layout, ResourceError, World};
use ahash::AHashMap;
use std::{any::{Any, TypeId}, ptr::NonNull};

// A resource is some shared data that will be accessed by multiple systems
// This resource cannot be removed from the systems. To be able to remove resources, we must implement the Removable trait as well
pub trait Resource: 'static {
    // Conversions to dynamic any
    fn as_any(&self) -> &dyn Any;
    fn as_any_mut(&mut self) -> &mut dyn Any;

    // This method will be called whenever we need to fetch the pointer of this resource from within the world
    fn fetch_ptr(world: &mut World) -> Result<NonNull<Self>, ResourceError> where Self: Sized {
        world
            .get_mut_unique::<Self>()
            .map(|r| NonNull::new(r as *mut Self).unwrap())
    }
}
