use std::any::TypeId;

use ahash::AHashMap;

use crate::{Events, Resource, ResourceError, StorageSet, Layout};


// The world is a container for multiple resources and events
// All the game engine logic is stored within the world, like ECS and Asset management
// Each World can be created using the builder pattern with the help of an App
#[derive(Default)]
pub struct World {
    resources: AHashMap<TypeId, Box<dyn Resource>>,
    events: Events,
}

impl World {
    // Get an immutable reference to the inner event handler
    // Even though this reference is immutable, we can still modify the events since it uses inner mutability
    pub fn events(&self) -> &Events {
        &self.events
    }

    // Get a mutable reference to the boxed resource from the set by casting it first
    pub(crate) fn fetch<T: Resource>(&mut self) -> Result<&mut T, ResourceError> {
        let boxed = self
            .resources
            .get_mut(&TypeId::of::<T>())
            .ok_or(ResourceError::missing::<T>())?;
        Ok(boxed.as_any_mut().downcast_mut::<T>().unwrap())
    }

    // Insert a new resource into the set (this requires the event set that we fetch from the world)
    pub fn insert<R: Resource>(&mut self, resource: R) {
        let mut boxed = Box::new(resource);
        boxed.inserted(&self.events);
        self.resources.insert(TypeId::of::<R>(), boxed);
    }

    // Remove a resouce from the set (if possible)
    // This returns true if we successfully deleted the resource
    pub fn remove<R: Resource>(&mut self) -> bool {
        if R::can_remove() {
            self.resources.remove(&TypeId::of::<R>()).is_some()
        } else {
            false
        }
    }

    // Fetch a tuple of certain resource handles from the set
    pub fn get_mut<'a, L: Layout<'a>>(&'a mut self) -> Result<L, ResourceError> {
        L::validate().map(|_| unsafe { L::fetch_unchecked(self) })?
    }

    // Check if a resource is contained within the set
    pub fn contains<R: Resource>(&self) -> bool {
        self.resources.contains_key(&TypeId::of::<R>())
    }

    // Get a set of all the inner storage resources
    pub(crate) fn storages<'a>(&'a mut self) -> StorageSet<'a> {
        StorageSet(self)
    }
}
