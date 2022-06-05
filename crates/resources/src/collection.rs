use std::{sync::{Arc, Mutex}, cell::RefCell, marker::PhantomData};
use slotmap::{SlotMap, DefaultKey};
use crate::Resource;

// A collection is a resource that keeps a list of unique data stored in memory for the lifetime of it's resource
#[derive(Resource)]
pub struct Collection<T: 'static>(SlotMap<DefaultKey, T>);

impl<T: 'static> Collection<T> {
    // Get an immutable reference to item
    fn get(&self, handle: Handle<T>) -> &T {
        self.0.get(handle.0).unwrap()
    }

    // Get a mutable reference to an item
    fn get_mut(&mut self, handle: Handle<T>) -> &mut T {
        self.0.get_mut(handle.0).unwrap()
    }

    // Insert an item to the collection, and return it's valid handle
    fn insert(&mut self, item: T) -> Handle<T> {
        let key = self.0.insert(item);
        Handle(key, Default::default())
    }

    // Remove the item using it's respective handle
    fn remove(&mut self, handle: Handle<T>) {
        self.0.remove(handle.0).unwrap();
    }
}

// A handle to an object that is stored within a collection
pub struct Handle<T>(DefaultKey, PhantomData<*const T>);