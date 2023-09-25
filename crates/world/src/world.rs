use ahash::AHashMap;
use thiserror::Error;
use std::{
    any::TypeId,
    cell::{Ref, RefCell, RefMut},
};
use crate::resource::{Resource, Read};

/// A world is a container for resources that are stored persistently throughout the game lifetime
/// Most systems will reference the internal resources directly to make use of parallelism, 
/// but if you wish to "dynamically" access resources you can access the world and use the
/// ``get`` and ``get_mut`` functions to fetch resources directly.
pub struct World(pub(crate) AHashMap<TypeId, RefCell<Box<dyn Resource>>>);

impl World {
    /*
#[derive(Error, Debug)]
pub enum WorldBorrowError {
    #[error("Resource is not present in the world")]
    NotPresent,

    #[error("{0}")]
    BorrowError(core::cell::BorrowError),
}

#[derive(Error, Debug)]
pub enum WorldBorrowMutError {
    #[error("Resource is not present in the world")]
    NotPresent,

    #[error("{0}")]
    BorrowMutError(core::cell::BorrowMutError),
}

    // Insert a new resource into the world
    pub fn insert<R: Resource>(&mut self, resource: R) {
        let id = TypeId::of::<R>();
        let returned = self.0.insert(id, RefCell::new(Box::new(resource)));
        if returned.is_some() {
            let name = pretty_type_name::pretty_type_name::<R>();
            log::warn!("Replaced resource {} since it was already present", name);
        }
    }

    // Get an immutable reference (read guard) to a resource
    pub fn get<R: Resource>(&self) -> Result<Read<R>, WorldBorrowError> {
        let cell = self
            .0
            .get(&TypeId::of::<R>())
            .ok_or(WorldBorrowError::NotPresent)?;
        let borrowed = cell.try_borrow().map_err(WorldBorrowError::BorrowError)?;
        let borrowed = Ref::map(borrowed, |boxed| {
            boxed.as_ref().as_any().downcast_ref::<R>().unwrap()
        });
        Ok(Read(borrowed))
    }

    // Get a mutable reference (write guard) to a resource
    pub fn get_mut<R: Resource>(&self) -> Result<Write<R>, WorldBorrowMutError> {
        let cell = self
            .0
            .get(&TypeId::of::<R>())
            .ok_or(WorldBorrowMutError::NotPresent)?;
        let borrowed = cell
            .try_borrow_mut()
            .map_err(WorldBorrowMutError::BorrowMutError)?;
        let borrowed = RefMut::map(borrowed, |boxed| {
            boxed.as_mut().as_any_mut().downcast_mut::<R>().unwrap()
        });
        Ok(Write(borrowed))
    }

    // Get an entry for a specific resource
    pub fn entry<R: Resource>(&mut self) -> Entry<'_, R> {
        Entry {
            world: self,
            _phantom: std::marker::PhantomData,
        }
    }

    // Remove a specific resource from the world
    pub fn remove<R: Resource>(&mut self) -> Option<R> {
        self.0.remove(&TypeId::of::<R>()).map(|cell| {
            let boxed = cell.into_inner();
            let any = boxed.into_any();
            let downcasted = any.downcast::<R>().unwrap();
            *downcasted
        })
    }

    // Check if a resource is present in the world
    pub fn contains<R: Resource>(&self) -> bool {
        self.0.contains_key(&TypeId::of::<R>())
    }
    */
}
