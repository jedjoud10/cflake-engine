use crate::{Events, Layout, ResourceError, World, FromWorld};
use ahash::AHashMap;
use std::{any::{Any, TypeId}, ptr::NonNull, marker::PhantomData};

// A resource is some shared data that will be accessed by multiple systems
// This resource cannot be removed from the systems. To be able to remove resources, we must implement the Removable trait as well
pub trait Resource: 'static {
    // Conversions to dynamic any
    fn as_any(&self) -> &dyn Any;
    fn as_any_mut(&mut self) -> &mut dyn Any;

    // This method will be called whenever we need to fetch the pointer of this resource from within the world
    fn fetch_ptr(world: &mut World) -> Result<NonNull<Self>, ResourceError> where Self: Sized {
        world
            .get_mut_unique::<Self>()
            .map(|r| NonNull::new(r as *mut Self).unwrap())
    }
}

// A resource entry is another way for the user to access unique resources from the world
// Entries allow us to insert default values if the actual underlying resource is missing
pub struct Entry<'a, R: Resource> {
    pub(crate) world: &'a mut World,
    pub(crate) _phantom: PhantomData<&'a mut R>,
}

impl<'a, R: Resource> Entry<'a, R> {
    // This will instantiate a new resource and insert it automatically into the world
    // This will only insert the resource if it is missing, and so, it will not duplicate it
    pub fn or_insert_from_world(self) -> &'a mut R where R: FromWorld {
        self.or_insert_with(|world| R::from_world(world))
    } 

    // This will return a mutable reference to the underlying resource if it exists
    // If the resource is missing, it will insert the given default value instead and use that are the return value
    // It is preferred to use or_insert_with instead, since it will lazyly evaluate the function
    pub fn or_insert(self, default: R) -> &'a mut R {
        self.or_insert_with(|_| default)
    }

    // This will return a mutable reference to the underlying resource if it exists
    // If the resource is missing, this will call the given function and insert the resource into the world
    pub fn or_insert_with<F: FnOnce(&mut World) -> R>(self, default: F) -> &'a mut R {
        if self.world.contains::<R>() {
            self.world.get_mut::<&mut R>().unwrap()
        } else {
            let resource = default(self.world);
            self.world.insert(resource);
            self.world.get_mut::<&mut R>().unwrap()
        }
    }

    // This will return a mutable reference to the underlying resource if it exists
    // If the resource is missing, this will automatically call the Default implementation of the resource and instantiate it, then insert it into the world
    pub fn or_default(self) -> &'a mut R where R: Default {
        self.or_insert_with(|_| Default::default())
    }
}