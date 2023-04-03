use parking_lot::{Mutex, RwLock};
use slotmap::{DefaultKey, SecondaryMap, SlotMap};

use std::{
    marker::PhantomData,
    ops::{Index, IndexMut},
    sync::{
        atomic::{AtomicBool, AtomicU32, Ordering},
        Arc,
    },
};

use crate::Handle;

use super::Weak;

pub(super) struct Trackers {
    pub(super) dropped: Mutex<Vec<DefaultKey>>,
    pub(super) counters: RwLock<SecondaryMap<DefaultKey, AtomicU32>>,
    pub(super) cleaned: AtomicBool,
}

// TODO: Rewrite tracking logic

pub struct Storage<T: 'static> {
    map: SlotMap<DefaultKey, Option<T>>,
    trackers: Arc<Trackers>,
}

impl<T: 'static> Default for Storage<T> {
    fn default() -> Self {
        Self {
            map: Default::default(),
            trackers: Arc::new(Trackers {
                dropped: Mutex::new(Default::default()),
                counters: RwLock::new(Default::default()),
                cleaned: AtomicBool::new(true),
            }),
        }
    }
}

impl<T: 'static> Storage<T> {
    // Insert a new value into the storage, returning a strong handle
    pub fn insert(&mut self, value: T) -> Handle<T> {
        self.clean();
        let key = self.map.insert(Some(value));
        self.trackers
            .counters
            .write()
            .insert(key, AtomicU32::new(1));

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
        if !self.trackers.cleaned.load(Ordering::Relaxed) {
            self.trackers.cleaned.store(true, Ordering::Relaxed);
            let mut dropped = self.trackers.dropped.lock();
            let mut counters = self.trackers.counters.write();
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
        let counters = self.trackers.counters.write();

        for count in counters.values() {
            if count.load(Ordering::Relaxed) != 0 {
                // TODO: Handle this
                //panic!("Cannot drop storage that has dangling handles");
            }
        }
    }
}
