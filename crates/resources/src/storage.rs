use ahash::AHashMap;
use slotmap::{DefaultKey, SecondaryMap, SlotMap};
use std::{cell::RefCell, marker::PhantomData, rc::Rc};

use crate::Resource;

// Keeps track of the number of handles per element
type Tracker = RefCell<AHashMap<DefaultKey, u32>>;

// A storage simply contains multiple values of the same type
// These values can then be acessed using handles. If a value has no handles, it will automatically get removed from the storage
pub struct Storage<T: 'static>(SlotMap<DefaultKey, T>, Rc<Tracker>);

impl<T: 'static> Storage<T> {
    // Insert a new value into the shared storage
    pub fn insert(&mut self, value: T) -> Handle<T> {
        let key = self.0.insert(value);
        self.1.borrow_mut().insert(key, 1);
        Handle {
            key: key,
            phantom_: Default::default(),
            tracker: self.1.clone(),
        }
    }

    // Get a value immutably
    pub fn get(&self, handle: &Handle<T>) -> &T {
        self.0.get(handle.key).unwrap()
    }

    // Get a value mutably
    pub fn get_mut(&mut self, handle: &Handle<T>) -> &mut T {
        self.0.get_mut(handle.key).unwrap()
    }

    // Try to get a value immutably (this will return None if the input is None)
    pub fn try_get(&self, handle: Option<&Handle<T>>) -> Option<&T> {
        handle.map(|h| self.get(h))
    }

    // Try to get a value mutably (this will return None if the input is None)
    pub fn try_get_mut(&mut self, handle: Option<&Handle<T>>) -> Option<&mut T> {
        handle.map(|h| self.get_mut(h))
    }
}

impl<T: 'static> Default for Storage<T> {
    fn default() -> Self {
        Self(Default::default(), Default::default())
    }
}

impl<T> Resource for Storage<T> {
    fn as_any(&self) -> &dyn std::any::Any {
        self
    }

    fn as_any_mut(&mut self) -> &mut dyn std::any::Any {
        self
    }

    fn pre_fetch(set: &mut crate::ResourceSet)
    where
        Self: Sized + 'static,
    {
        // Insert a default empty storage if it is non-existant
        if !set.contains::<Self>() {
            set.insert(Self::default())
        }
    }

    fn can_remove() -> bool {
        false
    }

    fn end_frame(&mut self) {
        let mut borrow = self.1.borrow_mut();
        borrow.retain(|key, count| {
            if *count == 0 {
                self.0.remove(*key).unwrap();
                false
            } else {
                true
            }
        });
    }
}

// A handle that will keep a certain value alive
// Handles can be cloned since we can share certain elements
pub struct Handle<T> {
    key: DefaultKey,
    tracker: Rc<Tracker>,
    phantom_: PhantomData<T>,
}

impl<T> PartialEq for Handle<T> {
    fn eq(&self, other: &Self) -> bool {
        self.key == other.key
    }
}

impl<T> Eq for Handle<T> {}

impl<T> Clone for Handle<T> {
    fn clone(&self) -> Self {
        Self {
            key: self.key.clone(),
            tracker: self.tracker.clone(),
            phantom_: self.phantom_.clone(),
        }
    }
}

impl<T> Drop for Handle<T> {
    fn drop(&mut self) {
        // Decrement the handle counter
        let mut tracker = self.tracker.borrow_mut();
        *tracker.get_mut(&self.key).unwrap() -= 1;
    }
}
