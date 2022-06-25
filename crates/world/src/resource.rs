use crate::{Events, Layout, ResourceError, StorageSet, World};
use ahash::AHashMap;
use std::any::{Any, TypeId};

// A resource is some shared data that will be accessed by multiple systems
pub trait Resource: 'static {
    // Conversions to dynamic any
    fn as_any(&self) -> &dyn Any;
    fn as_any_mut(&mut self) -> &mut dyn Any;

    // This function is called before we *try* to fetch the pointer for the specific resource
    fn fetch(world: &mut World)
    where
        Self: Sized,
    {
    }

    // This method will be called right before we insert the resource into the world
    fn inserted(&mut self, world: &mut World) {}

    // This tells us if we have the ability to remove the resource from the main set
    fn removable(world: &mut World) -> bool
    where
        Self: Sized,
    {
        true
    }
}
