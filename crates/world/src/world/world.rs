use crate::{
    world, Entry, Events, Init, Layout, Resource, ResourceError, Stage, StorageSetDescriptor,
    Update,
};
use ahash::AHashMap;
use std::any::TypeId;

// The world is a unique container for multiple resources
// All the game engine logic is stored within the world, like ECS and Asset management
// Each World can be created using the builder pattern with the help of an App
pub struct World {
    pub(crate) resources: AHashMap<TypeId, Box<dyn Resource>>,
}

// This is the main world state that the user can manually update to force the engine to stop running
#[derive(Resource)]
pub enum State {
    // This is the default state for frame 0
    Initializing,

    // This is the default state from frame 1 to frame n
    Running,

    // This is only set manually, by the user
    Stopped,
}

impl World {
    // Get a mutable reference to a singleboxed resource from the set by casting it first
    pub fn get_mut_unique<T: Resource>(&mut self) -> Result<&mut T, ResourceError> {
        let boxed = self
            .resources
            .get_mut(&TypeId::of::<T>())
            .ok_or(ResourceError::missing::<T>())?;
        Ok(boxed.as_any_mut().downcast_mut::<T>().unwrap())
    }

    // Insert a new resource into the set (this requires the event set that we fetch from the world)
    pub fn insert<R: Resource>(&mut self, resource: R) {
        let boxed = Box::new(resource);
        self.resources.insert(TypeId::of::<R>(), boxed);
    }

    // Remove a resouce from the set (if possible)
    // This returns true if we successfully deleted the resource
    pub fn remove<R: Resource>(&mut self) -> bool {
        self.resources.remove(&TypeId::of::<R>()).is_some()
    }

    // Fetch a tuple of certain resource handles from the set
    pub fn get_mut<'a, L: Layout<'a>>(&'a mut self) -> Result<L, ResourceError> {
        L::validate().map(|_| unsafe { L::fetch_unchecked(self) })?
    }

    // Check if a resource is contained within the set
    pub fn contains<R: Resource>(&self) -> bool {
        self.resources.contains_key(&TypeId::of::<R>())
    }

    // Get a resource entry that we can use to overwrite or insert missing resources with
    pub fn entry<'a, T: Resource>(&'a mut self) -> Entry<'a, T> {
        Entry {
            world: self,
            _phantom: Default::default(),
        }
    }
}

// This trait will be implemented for types that can be instantiated from the world
// An example of this would be the storage resources, since we require the world to create them and insert them
pub trait FromWorld {
    fn from_world(world: &mut World) -> Self;
}

// Global world system for cleaning and handling world state
pub fn system(events: &mut Events) {
    // At the end of every frame, we clean ALL the storages
    fn clean(world: &mut World) {
        let descriptor = world.get_mut::<&mut StorageSetDescriptor>().unwrap();
        for obj in descriptor.storages.iter() {
            obj.remove_dangling();
        }
    }

    // Insert the default world state event
    fn insert(world: &mut World) {
        world.insert(State::Initializing);
    }

    // Register the cleaning event (doesn't really matter *when* we execute it really)
    events
        .registry::<Update>()
        .insert_with(clean, Stage::new("storage clean").after("post user"))
        .unwrap();

    // Register the init state event
    events
        .registry::<Init>()
        .insert_with(insert, Stage::new("state insert").before("user"))
        .unwrap();
}
