use std::{marker::PhantomData, sync::Arc};

use super::{Handle, PipelineElemKey};
use crate::object::PipelineElement;

use parking_lot::Mutex;
use slotmap::SlotMap;

// A pipeline collection that contains multiple elements of the same type
pub struct PipelineCollection<T: PipelineElement> {
    // The inner storage
    inner: SlotMap<PipelineElemKey, T>,
    // Keep track of the elements that must be removed
    to_remove: Arc<Mutex<Vec<PipelineElemKey>>>,
}

impl<T: PipelineElement> Default for PipelineCollection<T> {
    fn default() -> Self {
        Self {
            inner: Default::default(),
            to_remove: Default::default(),
        }
    }
}

impl<T: PipelineElement> PipelineCollection<T> {
    // Update the collection, and remove any elements that have no longer have strong Handles
    pub fn dispose_dangling(&mut self) {
        let mut to_remove_locked = self.to_remove.lock();
        let to_remove = std::mem::take(&mut *to_remove_locked);
        for key in to_remove {
            // Silently ignores elements that have already been removed
            if let Some(removed) = self.inner.remove(key) {
                removed.disposed()
            }
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
    // Insert an element to the collection, returning it's specific handle
    pub fn insert(&mut self, value: T) -> Handle<T> {
        let key = self.inner.insert(value);

        Handle {
            key: Arc::new(key),
            to_remove: Some(self.to_remove.clone()),
            _phantom: PhantomData::default(),
        }
    }
}
