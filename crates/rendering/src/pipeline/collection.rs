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
    pub fn get(&self, handle: Handle<T>) -> Ref<Option<T>> {
        let inner = self.inner.borrow();
        Ref::map(inner, |slotmap| slotmap.get(handle.key))
    }
    // Mutably get an element
    pub fn get_mut(&self, handle: Handle<T>) -> RefMut<Option<T>> {
        let inner = self.inner.borrow_mut();
        RefMut::map(inner, |slotmap| slotmap.get(handle.key))
    }
    
    // Insert an element to the collection, returning it's specific handle
    pub fn insert(&mut self, value: T) -> Handle<T> {
        let inner = self.inner.borrow_mut();
        let key = inner.insert(value);
        Handle {
            inner: self.inner.clone(),
            key: Rc::new(key),
        }
    }
}

