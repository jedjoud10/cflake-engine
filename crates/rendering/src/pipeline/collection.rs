use std::{rc::Rc, cell::{RefCell, Ref, RefMut}, sync::Arc, marker::PhantomData};

use bitfield::AtomicSparseBitfield;
use parking_lot::Mutex;
use slotmap::SlotMap;
use crate::object::PipelineCollectionElement;
use super::{PipelineElemKey, Handle};


// A pipeline collection that contains multiple elements of the same type
pub(crate) type InnerPipelineCollection<T> = Rc<RefCell<SlotMap<PipelineElemKey, T>>>;
pub struct PipelineCollection<T: PipelineCollectionElement> {
    // The inner storage
    inner: InnerPipelineCollection<T>,
    // Keep track of the elements that must be removed
    to_remove: Arc<Mutex<Vec<PipelineElemKey>>>,
}

impl<T: PipelineCollectionElement> Default for PipelineCollection<T> {
    fn default() -> Self {
        Self { inner: Default::default(), to_remove: Default::default() }
    }
}

impl<T: PipelineCollectionElement> PipelineCollection<T> {
    // Update the collection, and remove any elements that have no longer have strong Handles
    pub fn dispose_dangling(&mut self) {
        let mut to_remove_locked = self.to_remove.lock();
        let to_remove = std::mem::take(&mut *to_remove_locked);
        let mut inner = self.inner.borrow_mut();
        for key in to_remove {
            // Silently ignores elements that have already been removed
            inner.remove(key);
        } 
    }
    // Get an element
    pub fn get(&self, handle: &Handle<T>) -> Option<Ref<T>> {
        let inner = self.inner.borrow();
        if !inner.contains_key(*handle.key) { return None; }        
        Some(Ref::map(inner, |slotmap| slotmap.get(*handle.key).unwrap()))
    }
    // Mutably get an element
    pub fn get_mut(&self, handle: &Handle<T>) -> Option<RefMut<T>> {
        let inner = self.inner.borrow_mut();
        if !inner.contains_key(*handle.key) { return None; }
        Some(RefMut::map(inner, |slotmap| slotmap.get_mut(*handle.key).unwrap()))
    }
    // Manually remove an element, though you should never do this since the Drop implementation on Handle<T> handles automatically removing unused elements
    pub fn remove(&mut self, handle: &Handle<T>) {
        let mut inner = self.inner.borrow_mut();
        // Silently ignores elements that have already been removed
        inner.remove(*handle.key);
    }    
    // Insert an element to the collection, returning it's specific handle
    pub fn insert(&mut self, value: T) -> Handle<T> {
        let inner_ = self.inner.clone();
        let mut inner = inner_.borrow_mut();
        let key = inner.insert(value);
        // Generate the OpenGL objects now
        let elem = inner.get_mut(key).unwrap();
        let handle = Handle {
            key: Arc::new(key),
            to_remove: Some(self.to_remove.clone()),
            _phantom: PhantomData::default(),
        };
        elem.added(self, handle.clone());  
        handle      
    }
}

