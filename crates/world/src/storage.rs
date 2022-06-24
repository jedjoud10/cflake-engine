use ahash::AHashMap;
use slotmap::{DefaultKey, SlotMap};
use std::{any::type_name, cell::RefCell, marker::PhantomData, rc::Rc};
use crate::{Resource, World};

// Keeps track of the number of handles per element
type Tracker = RefCell<AHashMap<DefaultKey, u32>>;

// A storage simply contains multiple elements of the same type
// These elements can then be acessed using handles. If a element has no handles, it will automatically get removed from the storage
pub struct Storage<T: 'static>(SlotMap<DefaultKey, T>, Rc<Tracker>);

impl<T: 'static> Storage<T> {
    // Insert a new element into the shared storage
    pub fn insert(&mut self, element: T) -> Handle<T> {
        let key = self.0.insert(element);
        self.1.borrow_mut().insert(key, 1);
        Handle {
            key,
            phantom_: Default::default(),
            tracker: self.1.clone(),
        }
    }

    // Get a element immutably
    pub fn get(&self, handle: &Handle<T>) -> &T {
        self.0.get(handle.key).unwrap()
    }

    // Get a element mutably
    pub fn get_mut(&mut self, handle: &Handle<T>) -> &mut T {
        self.0.get_mut(handle.key).unwrap()
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

    fn inserted(&mut self, events: &crate::Events) {   
        fn test(w: &mut World) {

        }
        let test = 0;
        let byref = move |world: &mut World| {
            dbg!(&test);
        };
        
        events.register(byref);
    }

    fn pre_fetch(world: &mut crate::World)
    where
        Self: Sized + 'static,
    {
        // Insert a default empty storage if it is non-existant
        if !world.contains::<Self>() {
            world.insert(Self::default())
        }
    }

    fn can_remove() -> bool {
        false
    }
}

// A handle that will keep a certain element alive
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

// Cloning the handle increments the reference count
impl<T> Clone for Handle<T> {
    fn clone(&self) -> Self {
        *self.tracker.borrow_mut().get_mut(&self.key).unwrap() += 1;
        Self {
            key: self.key,
            tracker: self.tracker.clone(),
            phantom_: self.phantom_,
        }
    }
}

// Dropping the handle decreases the reference count
impl<T> Drop for Handle<T> {
    fn drop(&mut self) {
        let mut tracker = self.tracker.borrow_mut();
        let value = tracker.get_mut(&self.key).unwrap().saturating_sub(1);
        *tracker.get_mut(&self.key).unwrap() = value;
    }
}

// A storage set is an abstraction over the resource set to allow for easier access to storages and their handles
pub struct StorageSet<'a>(pub(super) &'a mut World);

impl<'a> StorageSet<'a> {
    // Insert a new element into it's corresponding storage
    // This will automatically insert the storage resource if it does not exist yet
    pub fn insert<T: 'static>(&mut self, element: T) -> Handle<T> {
        let storage = self.0.get_mut::<&mut Storage<T>>().unwrap();
        storage.insert(element)
    }

    // This will get a reference to an element using it's handle
    pub fn get<T: 'static>(&mut self, handle: &Handle<T>) -> &T {
        let storage = self.0.get_mut::<&Storage<T>>().unwrap();
        storage.get(handle)
    }

    // This will get a mutable reference to an element using it's handle
    pub fn get_mut<T: 'static>(&mut self, handle: &Handle<T>) -> &mut T {
        let storage = self.0.get_mut::<&mut Storage<T>>().unwrap();
        storage.get_mut(handle)
    }
}
