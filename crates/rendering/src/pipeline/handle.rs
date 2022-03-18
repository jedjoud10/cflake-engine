use crate::object::PipelineElement;

use parking_lot::Mutex;
use slotmap::Key;
use std::{hash::Hash, marker::PhantomData, sync::Arc};

// A unique pipeline collection key
slotmap::new_key_type! {
    pub struct PipelineElemKey;
}

// A strong handle to a pipeline object. If there are 0 strong handles, the pipeline object will be deallocated (totally not stolen from Bevy)
pub struct Handle<T: PipelineElement> {
    pub(crate) key: Arc<PipelineElemKey>,
    pub(crate) to_remove: Option<Arc<Mutex<Vec<PipelineElemKey>>>>,
    pub(crate) _phantom: PhantomData<T>,
}

// Bruh derive moment
impl<T: PipelineElement> PartialEq for Handle<T> {
    fn ne(&self, other: &Self) -> bool {
        self.key != other.key
    }

    fn eq(&self, other: &Self) -> bool {
        self.key == other.key
    }
}
impl<T: PipelineElement> Eq for Handle<T> {}
impl<T: PipelineElement> Hash for Handle<T> {
    fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
        self.key.hash(state);
    }
}

// Sad
unsafe impl<T: PipelineElement> Send for Handle<T> {}
unsafe impl<T: PipelineElement> Sync for Handle<T> {}

impl<T: PipelineElement> std::fmt::Debug for Handle<T> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("Handle").field("key", &self.key).finish()
    }
}

impl<T: PipelineElement> Default for Handle<T> {
    fn default() -> Self {
        Self::null()
    }
}

impl<T: PipelineElement> Clone for Handle<T> {
    fn clone(&self) -> Self {
        Self {
            key: self.key.clone(),
            to_remove: self.to_remove.clone(),
            _phantom: PhantomData::default(),
        }
    }
}

impl<T: PipelineElement> Handle<T> {
    // Check if a handle is valid
    pub fn is_null(&self) -> bool {
        self.key.is_null()
    }
    // Create a new invalid handle
    pub fn null() -> Self {
        Self {
            key: Arc::new(PipelineElemKey::null()),
            to_remove: None,
            _phantom: PhantomData::default(),
        }
    }
    // Map the handle if it is invalid
    pub fn fallback_to<'a>(&'a self, default: &'a Self) -> &'a Self {
        if !self.is_null() {
            self
        } else {
            default
        }
    }
}

impl<T: PipelineElement> Drop for Handle<T> {
    // Remove the element if this is the last strong handle it has
    fn drop(&mut self) {
        if let Some(to_remove) = &self.to_remove {
            let strong_count = Arc::strong_count(&self.key);
            if strong_count == 1 {
                // Remove the element that this Handle referred to
                let mut inner = to_remove.lock();
                inner.push(*self.key.as_ref());
            }
        }
    }
}
