use crate::{resource::Resource, system::System};
use ahash::AHashMap;
use atomic_refcell::{AtomicRefCell, AtomicRef, AtomicRefMut};
use std::{any::TypeId, sync::Arc, marker::PhantomData};

/// A world is a container for resources that are stored persistently throughout the game lifetime
pub struct World(pub(crate) AHashMap<TypeId, Arc<AtomicRefCell<Box<dyn Resource>>>>);

/// A WorldView allows you to access immutable/mutable resources from the world in parallel with other systems
/// You can access resources that you are allowed to access given by your systems' "access" mask
/// If you try accessing a resource that you are not allowed to, the system will panic
pub struct WorldView<'a> {
    resources: &'a AHashMap<TypeId, Arc<AtomicRefCell<Box<dyn Resource>>>>,
}

impl WorldView<'_> {
    /// Get a resource immutably
    /// Panics if the SystemData does not allow it
    pub fn get<T: Resource>(&self) -> AtomicRef<T> {
        todo!()
    }
    
    /// Get a resource mutably
    /// Panics if the SystemData does not allow it
    pub fn get_mut<T: Resource>(&self) -> AtomicRefMut<T> {
        todo!()
    }

    /// Add a resource to the world
    /// Internally stored in a command buffer first
    /// Panics if the SystemData does not allow it
    pub fn insert<T: Resource>(&mut self, resource: T) {
        todo!()
    }

    /// Create an entry for a resource to initialize it
    /// Internally stored in a command buffer first
    /// Panics if the SystemData does not allow it
    
    /// Remove a resource from the world
    /// Panics if the SystemData does not allow it
    fn remove<T: Resource>(&mut self) -> T {
        todo!()
    }
}

impl World {}