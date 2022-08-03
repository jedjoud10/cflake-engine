use slotmap::{DefaultKey, SecondaryMap, SlotMap};

use std::{
    cell::{Cell, RefCell},
    marker::PhantomData,
    rc::Rc, ops::{Index, IndexMut},
};

struct Trackers {
    dropped: RefCell<Vec<DefaultKey>>,
    counters: RefCell<SecondaryMap<DefaultKey, u32>>,
    cleaned: Cell<bool>,
}

// Storages automatically get inserted into the world when we try to access them, so we must write that custom logic when implementing the resource trait
pub struct Storage<T: 'static> {
    map: SlotMap<DefaultKey, T>,
    trackers: Rc<Trackers>,
}

impl<T: 'static> Default for Storage<T> {
    fn default() -> Self {
        Self {
            map: Default::default(),
            trackers: Rc::new(Trackers {
                dropped: RefCell::new(Default::default()),
                counters: RefCell::new(Default::default()),
                cleaned: Cell::new(true),
            }),
        }
    }
}

impl<T: 'static> Storage<T> {
    // Insert a new value into the storage, returning it's handle
    pub fn insert(&mut self, value: T) -> Handle<T> {
        self.clean();
        let key = self.map.insert(value);
        self.trackers.counters.borrow_mut().insert(key, 1);

        Handle {
            _phantom: PhantomData::default(),
            untyped: UntypedHandle {
                trackers: self.trackers.clone(),
                key
            }
        }
    }

    // Get an immutable reference to a value using it's a handle
    pub fn get(&self, handle: &Handle<T>) -> &T {
        self.map.get(handle.untyped.key).unwrap()
    }

    // Get a mutable reference to a value using it's handle
    pub fn get_mut(&mut self, handle: &Handle<T>) -> &mut T {
        self.map.get_mut(handle.untyped.key).unwrap()
    }

    // Clean the storage of any dangling values. This will keep the same memory footprint as before
    pub fn clean(&mut self) {
        if !self.trackers.cleaned.get() {
            self.trackers.cleaned.set(true);
            let mut dropped = self.trackers.dropped.borrow_mut();
            let mut counters = self.trackers.counters.borrow_mut();
            for i in dropped.drain(..) {
                self.map.remove(i);
                counters.remove(i);
            }
        }
    }
}

impl<T: 'static> Index<Handle<T>> for Storage<T> {
    type Output = T;

    fn index(&self, index: Handle<T>) -> &Self::Output {
        self.get(&index)
    }
}

impl<T: 'static> IndexMut<Handle<T>> for Storage<T> {
    fn index_mut(&mut self, index: Handle<T>) -> &mut Self::Output {
        self.get_mut(&index)
    }
}

impl<T: 'static> Index<&'_ Handle<T>> for Storage<T> {
    type Output = T;

    fn index(&self, index: &Handle<T>) -> &Self::Output {
        self.get(index)
    }
}

impl<T: 'static> IndexMut<&'_ Handle<T>> for Storage<T> {
    fn index_mut(&mut self, index: &Handle<T>) -> &mut Self::Output {
        self.get_mut(index)
    }
}

impl<T: 'static> Drop for Storage<T> {
    fn drop(&mut self) {
        let counters = self.trackers.counters.borrow();

        for count in counters.values() {
            if *count != 0 {
                // TODO: Handle this
                //panic!("Cannot drop storage that has dangling handles");
            }
        }
    }
}

// UntypedHandle is a handle that will keep a special value stored within a storage alive until the last handle gets dropped
pub struct UntypedHandle {
    trackers: Rc<Trackers>,
    key: DefaultKey,
}

impl PartialEq for UntypedHandle {
    fn eq(&self, other: &Self) -> bool {
        self.key == other.key
    }
}

impl Eq for UntypedHandle {}

impl UntypedHandle {
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
impl Clone for UntypedHandle {
    fn clone(&self) -> Self {
        unsafe {
            self.increment_count();
        }
        Self {
            trackers: self.trackers.clone(),
            key: self.key,
        }
    }
}

// Dropping the handle will decrease the reference count of that handle
// If we drop the last valid handle, then the stored value will get dropped
impl Drop for UntypedHandle {
    fn drop(&mut self) {
        // If the counter reaches 0, it means that we must drop the inner value
        if unsafe { self.decrement_count() } == 0 {
            self.trackers.dropped.borrow_mut().push(self.key);
            self.trackers.cleaned.set(false);
        }
    }
}

// A handle is what keeps the values within Storage<T> alive
pub struct Handle<T: 'static> {
    _phantom: PhantomData<*mut T>,
    untyped: UntypedHandle,
}

impl<T: 'static> PartialEq for Handle<T> {
    fn eq(&self, other: &Self) -> bool {
        self.untyped == other.untyped
    }
}

impl<T: 'static> Eq for Handle<T> {}

impl<T: 'static> Handle<T> {
    // Get the current reference count for this handle
    pub fn count(&self) -> u32 {
        self.untyped.count()
    }

    // Overwrite the current reference counted value directly
    pub unsafe fn set_count(&self, count: u32) {
        self.untyped.set_count(count);
    }

    // This will manually incremememnt the underlying reference counter
    pub unsafe fn increment_count(&self) -> u32 {
        self.untyped.increment_count()
    }

    // This will manually decrement the underlying reference counter
    pub unsafe fn decrement_count(&self) -> u32 {
        self.untyped.decrement_count()
    }

    // Get the inner untyped handle
    pub fn untyped(&self) -> &UntypedHandle {
        &self.untyped
    }
}

// Cloning the handle will increase the reference count of that handle
impl<T: 'static> Clone for Handle<T> {
    fn clone(&self) -> Self {
        Self {
            _phantom: PhantomData::default(),
            untyped: self.untyped.clone(),
        }
    }
}