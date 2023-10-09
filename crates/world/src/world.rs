use crate::{resource::{Resource, Entry}, system::System};
use ahash::AHashMap;
use atomic_refcell::{AtomicRefCell, AtomicRef, AtomicRefMut};
use std::{any::TypeId, sync::Arc, marker::PhantomData, cell::{RefCell, Ref, RefMut}};

/// A world is a container for resources that are stored persistently throughout the game lifetime
#[derive(Default)]
pub struct World(pub(crate) AHashMap<TypeId, RefCell<Box<dyn Resource>>>);

impl World {
    /// Insert a new resource into the world
    pub fn insert<R: Resource>(&mut self, resource: R) {
        let name = std::any::type_name::<R>();
        let id = TypeId::of::<R>();
        let boxed = Box::new(resource) as Box<dyn Resource>;
        log::trace!("Add resource {name} to world");
        let returned = self.0.insert(id, RefCell::new(boxed));
        if returned.is_some() {
            log::warn!("Replaced resource {} since it was already present", name);
        }
    }

    /// Get an immutable reference (read guard) to a resource
    pub fn get<R: Resource>(&self) -> Option<Ref<R>> {
        let cell = self
            .0
            .get(&TypeId::of::<R>())?;
        Some(Ref::map(cell.borrow(), |boxed: &Box<dyn Resource>|
            boxed.as_ref().as_any().downcast_ref::<R>().unwrap()
        ))
    }

    /// Get a mutable reference (write guard) to a resource
    pub fn get_mut<R: Resource>(&self) -> Option<RefMut<R>> {
        let cell = self
            .0
            .get(&TypeId::of::<R>())?;
        Some(RefMut::map(cell.borrow_mut(), |boxed| 
            boxed.as_mut().as_any_mut().downcast_mut::<R>().unwrap()
        ))
    }

    /// Get an entry for a specific resource
    pub fn entry<R: Resource>(&mut self) -> Entry<'_, R> {
        Entry {
            world: self,
            _phantom: std::marker::PhantomData,
        }
    }

    /// Remove a specific resource from the world
    pub fn remove<R: Resource>(&mut self) -> Option<R> {
        self.0.remove(&TypeId::of::<R>()).map(|cell| {
            let boxed = cell.into_inner();
            let any = boxed.into_any();
            let downcasted = any.downcast::<R>().unwrap();
            *downcasted
        })
    }

    /// Check if a resource is present in the world
    pub fn contains<R: Resource>(&self) -> bool {
        self.0.contains_key(&TypeId::of::<R>())
    }
}