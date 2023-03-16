use slotmap::{DefaultKey, SecondaryMap, SlotMap};

use std::{
    cell::{Cell, RefCell},
    marker::PhantomData,
    ops::{Index, IndexMut},
    rc::Rc,
};

use crate::Handle;

use super::Weak;

pub(super) struct Trackers {
    pub(super) dropped: RefCell<Vec<DefaultKey>>,
    pub(super) counters: RefCell<SecondaryMap<DefaultKey, u32>>,
    pub(super) cleaned: Cell<bool>,
}

// TODO: Rewrite tracking logic

pub struct Storage<T: 'static> {
    map: SlotMap<DefaultKey, Option<T>>,
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
    // Insert a new value into the storage, returning a strong handle
    pub fn insert(&mut self, value: T) -> Handle<T> {
        self.clean();
        let key = self.map.insert(Some(value));
        self.trackers.counters.borrow_mut().insert(key, 1);

        Handle {
            _phantom: PhantomData::default(),
            trackers: self.trackers.clone(),
            key,
        }
    }

    // Get an immutable reference to a value using a strong handle
    pub fn get(&self, handle: &Handle<T>) -> &T {
        self.map.get(handle.key).unwrap().as_ref().unwrap()
    }

    // Try to get an immutable reference to a value using a weak handle
    pub fn try_get(&self, weak: &Weak<T>) -> Option<&T> {
        let slot = self.map.get(weak.key)?;
        let initialized = slot.as_ref()?;
        Some(initialized)
    }

    // Get a mutable reference to a value using a strong handle
    pub fn get_mut(&mut self, handle: &Handle<T>) -> &mut T {
        self.map.get_mut(handle.key).unwrap().as_mut().unwrap()
    }

    // Try to get a mutable reference to a value using a weak handle
    pub fn try_get_mut(&mut self, weak: &Weak<T>) -> Option<&mut T> {
        let slot = self.map.get_mut(weak.key)?;
        let initialized = slot.as_mut()?;
        Some(initialized)
    }

    // Get an immutable iterator over all the values in the storage
    pub fn iter(&self) -> impl Iterator<Item = &T> {
        self.map.iter().filter_map(|(_, x)| x.as_ref())
    }

    // Get a mutable iterator over all the values in the storage
    pub fn iter_mut(&mut self) -> impl Iterator<Item = &mut T> {
        self.map.iter_mut().filter_map(|(_, x)| x.as_mut())
    }

    // Clean the storage of any values that do not have any strong handles any more
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