use std::{rc::Rc, cell::{RefCell, Ref, RefMut}};

use slotmap::SlotMap;

use super::{PipelineElemKey, Handle};

// A pipeline collection that contains multiple elements of the same type
// This can only be accessed on the main thread
pub(crate) type InnerPipelineCollection<T> = Rc<RefCell<SlotMap<PipelineElemKey, T>>>;

pub struct PipelineCollection<T> {
    // The inner storage
    inner: InnerPipelineCollection<T>,
}

impl<T> PipelineCollection<T> {
    // Get an element
    pub fn get(&self, handle: Handle<T>) -> Option<Ref<T>> {
        let inner = self.inner.borrow();
        if !inner.contains_key(*handle.key) { return None; }        
        Some(Ref::map(inner, |slotmap| slotmap.get(*handle.key).unwrap()))
    }
    // Mutably get an element
    pub fn get_mut(&self, handle: Handle<T>) -> Option<RefMut<T>> {
        let inner = self.inner.borrow_mut();
        if !inner.contains_key(*handle.key) { return None; }
        Some(RefMut::map(inner, |slotmap| slotmap.get_mut(*handle.key).unwrap()))
    }
    
    // Insert an element to the collection, returning it's specific handle
    pub fn insert(&mut self, value: T) -> Handle<T> {
        let mut inner = self.inner.borrow_mut();
        let key = inner.insert(value);
        Handle {
            inner: Some(self.inner.clone()),
            key: Rc::new(key),
        }
    }
}

