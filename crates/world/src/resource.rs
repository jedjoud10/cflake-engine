use crate::{Layout, ResourceError, StorageSet, Events, World};
use ahash::AHashMap;
use std::any::{Any, TypeId};

// A resource is some shared data that will be accessed by multiple systems
pub trait Resource: 'static {
    // Bruh conversions
    fn as_any(&self) -> &dyn Any;
    fn as_any_mut(&mut self) -> &mut dyn Any;

    // A function that will be called whenever we successfully insert a resource into the world
    fn inserted(&mut self, events: &Events) {}

    // A function that will be called right before the resource gets fetch
    fn pre_fetch(_world: &mut World)
    where
        Self: Sized + 'static,
    {
    }

    // A function that hints the resource set if it can remove the resource or not
    fn can_remove() -> bool
    where
        Self: Sized,
    {
        true
    }
}
