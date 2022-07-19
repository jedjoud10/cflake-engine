use slotmap::{SlotMap, DefaultKey, SecondaryMap};

use crate::{self as world};
use crate::{FromWorld, Resource};
use std::{
    cell::{Cell, RefCell, UnsafeCell},
    marker::PhantomData,
    mem::ManuallyDrop,
    ptr::NonNull,
    rc::Rc,
};

struct Trackers {
    dropped: RefCell<Vec<DefaultKey>>,
    counters: RefCell<SecondaryMap<DefaultKey, u32>>,
}

// Storages automatically get inserted into the world when we try to access them, so we must write that custom logic when implementing the resource trait
pub struct Storage<T: 'static> {
    map: SlotMap<DefaultKey, T>,
    trackers: Rc<Trackers>,
}

impl<T: 'static> Default for Storage<T> {
    fn default() -> Self {
        Self { map: Default::default(), trackers: Rc::new(
            Trackers {
                dropped: RefCell::new(Default::default()),
                counters: RefCell::new(Default::default())
            })
        }
    }
}

impl<T: 'static> Storage<T> {
    // Insert a new value into the storage, returning it's handle
    pub fn insert(&mut self, value: T) -> Handle<T> {
        let key = self.map.insert(value);
        self.trackers.counters.borrow_mut().insert(key, 1).unwrap();

        Handle {
            _phantom: PhantomData::default(),
            trackers: self.trackers.clone(),
            key,
        }
    }
    
    // Get an immutable reference to a value using it's a handle
    pub fn get(&self, handle: &Handle<T>) -> &T {
        self.map.get(handle.key).unwrap()
    }
    
    // Get a mutable reference to a value using it's handle
    pub fn get_mut(&mut self, handle: &Handle<T>) -> &mut T {
        self.map.get_mut(handle.key).unwrap()
    }

    // Clean the storage of any dangling values
    pub fn clean(&mut self) {
        let mut dropped = self.trackers.dropped.borrow_mut();
        let mut counters = self.trackers.counters.borrow_mut();
        for i in dropped.drain(..) {
            self.map.remove(i);
            counters.remove(i);
        }
    }
}


// A handle is what keeps the values within Storage<T> alive
pub struct Handle<T: 'static> {
    _phantom: PhantomData<*mut T>,
    trackers: Rc<Trackers>,
    key: DefaultKey,
}

impl<T: 'static> PartialEq for Handle<T> {
    fn eq(&self, other: &Self) -> bool {
        self.key == other.key
    }
}

impl<T: 'static> Eq for Handle<T> {}

impl<T: 'static> Handle<T> {
    // Get the current reference count for this handle
    pub fn count(&self) -> u32 {
        *self.trackers.counters.borrow().get(self.key).unwrap()
    }

    // Overwrite the current reference counted value directly
    pub unsafe fn set_count(&self, count: u32) {
        let mut borrowed = self.trackers.counters.borrow_mut();
        *borrowed.get_mut(self.key).unwrap() = count;
    }

    // This will manually incremememnt the underlying reference counter
    pub unsafe fn increment_count(&self) -> u32 {
        let value = self.count().saturating_add(1);
        self.set_count(value);
        value
    }

    // This will manually decrement the underlying reference counter
    pub unsafe fn decrement_count(&self) -> u32 {
        let value = self.count().saturating_sub(1);
        self.set_count(value);
        value
    }
}

// Cloning the handle will increase the reference count of that handle
impl<T: 'static> Clone for Handle<T> {
    fn clone(&self) -> Self {
        unsafe {
            self.increment_count();
        }
        Self {
            _phantom: PhantomData::default(),
            trackers: self.trackers.clone(),
            key: self.key.clone(),
        }
    }
}

// Dropping the handle will decrease the reference count of that handle
// If we drop the last valid handle, then the stored value will get dropped
impl<T: 'static> Drop for Handle<T> {
    fn drop(&mut self) {
        // If the counter reaches 0, it means that we must drop the inner value
        if unsafe { self.decrement_count() } == 0 {
            self.trackers.dropped.borrow_mut().push(self.key);
        }
    }
}

impl<T: 'static> Default for Handle<T> {
    fn default() -> Self {
        Self { _phantom: Default::default(), trackers: todo!(), key: Default::default() }
    }
}