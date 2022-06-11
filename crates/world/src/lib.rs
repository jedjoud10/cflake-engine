pub use resources;
use resources::{Layout, Resource, ResourceError, ResourceSet};
use std::any::TypeId;

// The world is a container for multiple resources
// All the game engine logic is stored within the world, like ECS and Asset management
// Each World can be created using the builder pattern with the help of an App
#[derive(Default)]
pub struct World(ResourceSet);

impl World {
    // Insert a new resource into the world
    pub fn insert<R: Resource>(&mut self, resource: R) {
        self.0.insert::<R>(resource);
    }

    // Remove a resouce from the world
    pub fn remove<R: Resource>(&mut self) {
        self.0.remove::<R>();
    }

    // Fetch a tuple of certain resource handles from the world
    pub fn get_mut<'a, L: Layout<'a>>(&'a mut self) -> Result<L, ResourceError> {
        self.0.get_mut::<'a, L>()
    }
}