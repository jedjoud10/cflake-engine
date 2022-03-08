use std::{
    cell::{Ref, RefCell, RefMut},
    marker::PhantomData,
    rc::Rc,
    sync::Arc,
};

use super::{Handle, PipelineElemKey};
use crate::object::PipelineCollectionElement;
use bitfield::AtomicSparseBitfield;
use parking_lot::Mutex;
use slotmap::SlotMap;

// A pipeline collection that contains multiple elements of the same type
pub struct PipelineCollection<T: PipelineCollectionElement> {
    // The inner storage
    inner: SlotMap<PipelineElemKey, T>,
    // Keep track of the elements that must be removed
    to_remove: Arc<Mutex<Vec<PipelineElemKey>>>,
}

impl<T: PipelineCollectionElement> Default for PipelineCollection<T> {
    fn default() -> Self {
        Self {
            inner: Default::default(),
            to_remove: Default::default(),
        }
    }
}

impl<T: PipelineCollectionElement> PipelineCollection<T> {
    // Update the collection, and remove any elements that have no longer have strong Handles
    pub fn dispose_dangling(&mut self) {
        let mut to_remove_locked = self.to_remove.lock();
        let to_remove = std::mem::take(&mut *to_remove_locked);
        for key in to_remove {
            // Silently ignores elements that have already been removed
            if let Some(removed) = self.inner.remove(key) { removed.disposed() }
        }
    }
    // Iter
    pub fn iter(&self) -> impl Iterator<Item = (PipelineElemKey, &T)> {
        self.inner.iter()
    }
    pub fn iter_mut(&mut self) -> impl Iterator<Item = (PipelineElemKey, &mut T)> {
        self.inner.iter_mut()
    }
    // Get an element
    pub fn get(&self, handle: &Handle<T>) -> Option<&T> {
        self.inner.get(*handle.key)
    }
    // Mutably get an element
    pub fn get_mut(&mut self, handle: &Handle<T>) -> Option<&mut T> {
        self.inner.get_mut(*handle.key)
    }
    // Manually remove an element, though you should never do this since the Drop implementation on Handle<T> handles automatically removing unused elements
    pub fn remove(&mut self, handle: &Handle<T>) {
        // Silently ignores elements that have already been removed
        let removed = self.inner.remove(*handle.key);
        // Remember to dispose
        if let Some(removed) = removed { removed.disposed() }
    }
    // Insert an element to the collection, returning it's specific handle
    pub fn insert(&mut self, value: T) -> Handle<T> {
        let key = self.inner.insert(value);
        // Generate the OpenGL objects now
        let elem = self.inner.get_mut(key).unwrap();
        let handle = Handle {
            key: Arc::new(key),
            to_remove: Some(self.to_remove.clone()),
            _phantom: PhantomData::default(),
        };
        elem.added(&handle);
        handle
    }
}
