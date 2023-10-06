use crate::{resource::{Resource, Entry}, system::System};
use ahash::AHashMap;
use atomic_refcell::{AtomicRefCell, AtomicRef, AtomicRefMut};
use std::{any::TypeId, sync::Arc, marker::PhantomData, cell::{RefCell, Ref, RefMut}};

/// A world is a container for resources that are stored persistently throughout the game lifetime
#[derive(Default)]
pub struct World(pub(crate) AHashMap<TypeId, RefCell<Box<dyn Resource>>>);

impl World {
    // Insert a new resource into the world
    pub fn insert<R: Resource>(&mut self, resource: R) {
        let id = TypeId::of::<R>();
        let returned = self.0.insert(id, RefCell::new(Box::new(resource)));
        /*
        if returned.is_some() {
            let name = pretty_type_name::pretty_type_name::<R>();
            log::warn!("Replaced resource {} since it was already present", name);
        }
        */
    }

    // Get an immutable reference (read guard) to a resource
    pub fn get<R: Resource>(&self) -> Ref<R> {
        let cell = self
            .0
            .get(&TypeId::of::<R>())
            .unwrap();
        Ref::map(cell.borrow(), |x| x.as_any().downcast_ref::<R>().unwrap())
    }

    // Get a mutable reference (write guard) to a resource
    pub fn get_mut<R: Resource>(&self) -> RefMut<R> {
        let cell = self
            .0
            .get(&TypeId::of::<R>())
            .unwrap();
        RefMut::map(cell.borrow_mut(), |x| x.as_any_mut().downcast_mut::<R>().unwrap())
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
}

impl World {}