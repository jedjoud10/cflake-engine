use super::{Resource, Write};
use crate::world::World;
use std::marker::PhantomData;

/// A resource entry is another way for the user to access unique resources from the world
/// Entries allow us to insert default values if the actual underlying resource is missing
pub struct Entry<'a, R: Resource> {
    pub(crate) world: &'a mut World,
    pub(crate) _phantom: PhantomData<R>,
}

impl<'a, R: Resource> Entry<'a, R> {
    /// This will return a mutable reference to the underlying resource if it exists
    /// If the resource is missing, it will insert the given default value instead and use that are the return value
    /// It is preferred to use or_insert_with instead, since it will lazyly evaluate the function
    pub fn or_insert(self, default: R) -> Write<'a, R> {
        self.or_insert_with(|_| default)
    }

    /// This will return a mutable reference to the underlying resource if it exists
    /// If the resource is missing, this will call the given function and insert the resource into the world
    pub fn or_insert_with<F: FnOnce(&World) -> R>(self, default: F) -> Write<'a, R> {
        todo!()
        /*
        if self.world.contains::<R>() {
            self.world.get_mut::<R>().unwrap()
        } else {
            let resource = default(self.world);
            self.world.insert(resource);
            self.world.get_mut::<R>().unwrap()
        }
        */
    }

    /// This will return a mutable reference to the underlying resource if it exists
    /// If the resource is missing, this will automatically call the Default implementation of the resource and instantiate it, then insert it into the world
    pub fn or_default(self) -> Write<'a, R>
    where
        R: Default,
    {
        self.or_insert_with(|_| Default::default())
    }
}
