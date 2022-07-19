use crate::{
    Events, Init, Resource, Stage,
    Update, Read, Write,
};
use ahash::AHashMap;
use std::{any::{TypeId, Any}, cell::{RefCell, Ref, RefMut}};

// The world is a unique container for multiple resources like ECS and assets
// Each World can be created using the builder pattern with the help of an App
pub struct World(pub(crate) AHashMap<TypeId, RefCell<Box<dyn Resource>>>);

// This is the main world state that the user can manually update to force the engine to stop running
pub enum State {
    // This is the default state for frame 0
    Initializing,

    // This is the default state from frame 1 to frame n
    Running,

    // This is only set manually, by the user
    Stopped,
}

impl World {
    // Insert a new resource into the world
    pub fn insert<R: Resource>(&mut self, resource: R) -> Option<()> {
        let id = TypeId::of::<R>();
        if !self.0.contains_key(&id) {
            self.0.insert(id, RefCell::new(Box::new(resource)));
            Some(())
        } else {
            None
        }
    }

    // Get an immutable reference (read guard) to a resource
    pub fn get<R: Resource>(&self) -> Option<Read<R>> {
        self.0.get(&TypeId::of::<R>()).map(|cell| {
            let borrowed = cell.borrow();
            let borrowed = Ref::map(borrowed, |boxed| boxed.as_any().downcast_ref::<R>().unwrap());
            Read(borrowed)
        })
    }

    // Get a mutable reference (write guard) to a resource
    pub fn get_mut<R: Resource>(&self) -> Option<Write<R>> {
        self.0.get(&TypeId::of::<R>()).map(|cell| {
            let borrowed = cell.borrow_mut();
            let borrowed = RefMut::map(borrowed, |boxed| boxed.as_any_mut().downcast_mut::<R>().unwrap());
            Write(borrowed)
        })
    }

    // Remove a resource from the world
    pub fn remove<R: Resource>(&mut self) -> Option<()> {
        self.0.remove(&TypeId::of::<R>()).map(|_| ())
    }

    // Check if a resource is present in the world
    pub fn contains<R: Resource>(&self) -> bool {
        self.0.contains_key(&TypeId::of::<R>())
    }
}

// This trait will be implemented for types that can be instantiated from the world
// An example of this would be the storage resources, since we require the world to create them and insert them
pub trait FromWorld {
    fn from_world(world: &mut World) -> Self;
}

// Global world system for cleaning and handling world state
pub fn system(events: &mut Events) {
    // Insert the default world state event
    fn insert(world: &mut World) {
        world.insert(State::Initializing);
    }

    // Register the init state event
    events
        .registry::<Init>()
        .insert_with(insert, Stage::new("state insert").before("user"))
        .unwrap();
}
