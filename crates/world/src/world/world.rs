use crate::{
    user, Entry, Read, Resource, System, WorldBorrowError,
    WorldBorrowMutError, Write,
};
use ahash::AHashMap;
use std::{
    any::TypeId,
    cell::{Ref, RefCell, RefMut},
};

// The world is a unique container for multiple resources like ECS and assets
// Each World can be created using the builder pattern with the help of an App
// TODO: Maybe replace this with DashMap to allow non mutable insertions?
pub struct World(
    pub(crate) AHashMap<TypeId, RefCell<Box<dyn Resource>>>,
);

// This is the main world state that the user can manually update to force the engine to stop running
#[derive(Default, Clone, Copy, PartialEq, Eq)]
pub enum State {
    // This is the default state for frame 0
    #[default]
    Initializing,

    // This is the default state from frame 1 to frame n
    Running,

    // This is only set manually, by the user
    Stopped,
}

impl World {
    // Insert a new resource into the world
    pub fn insert<R: Resource>(&self, resource: R) {
        let id = TypeId::of::<R>();
        let returned =
            self.0.insert(id, RefCell::new(Box::new(resource)));
        if returned.is_some() {
            let name = pretty_type_name::pretty_type_name::<R>();
            log::warn!(
                "Replaced resource {} since it was already present",
                name
            );
        }
    }

    // Get an immutable reference (read guard) to a resource
    pub fn get<R: Resource>(
        &self,
    ) -> Result<Read<R>, WorldBorrowError> {
        let cell = self
            .0
            .get(&TypeId::of::<R>())
            .ok_or(WorldBorrowError::NotPresent)?;
        let borrowed = cell
            .try_borrow()
            .map_err(WorldBorrowError::BorrowError)?;
        let borrowed = Ref::map(borrowed, |boxed| {
            boxed.as_ref().as_any().downcast_ref::<R>().unwrap()
        });
        Ok(Read(borrowed))
    }

    // Get a mutable reference (write guard) to a resource
    pub fn get_mut<R: Resource>(
        &self,
    ) -> Result<Write<R>, WorldBorrowMutError> {
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
    pub fn entry<'a, R: Resource>(&'a self) -> Entry<'a, R> {
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
}

// Global world system for cleaning and handling world state
pub fn system(system: &mut System) {
    system
        .insert_init(|world: &mut World| {
            world.insert(State::default());
        })
        .before(user);
}
