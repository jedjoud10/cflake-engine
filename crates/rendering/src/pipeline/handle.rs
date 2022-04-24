use crate::object::Object;

use parking_lot::Mutex;
use slotmap::Key;
use std::{hash::Hash, marker::PhantomData, sync::Arc};

// A unique pipeline collection key
slotmap::new_key_type! {
    pub struct PipelineElemKey;
}

// A strong handle to a pipeline object. If there are 0 strong handles, the pipeline object will be deallocated (totally not stolen from Bevy)
pub struct Handle<T: Object> {
    pub(crate) key: Arc<PipelineElemKey>,
    pub(crate) to_remove: Option<Arc<Mutex<Vec<PipelineElemKey>>>>,
    pub(crate) _phantom: PhantomData<T>,
}

// Bruh derive moment
impl<T: Object> PartialEq for Handle<T> {
    fn ne(&self, other: &Self) -> bool {
        self.key != other.key
    }

    fn eq(&self, other: &Self) -> bool {
        self.key == other.key
    }
}
impl<T: Object> Eq for Handle<T> {}
impl<T: Object> Hash for Handle<T> {
    fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
        self.key.hash(state);
    }
}

// Sad
unsafe impl<T: Object> Send for Handle<T> {}
unsafe impl<T: Object> Sync for Handle<T> {}

impl<T: Object> std::fmt::Debug for Handle<T> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("Handle").field("key", &self.key).finish()
    }
}

impl<T: Object> Default for Handle<T> {
    fn default() -> Self {
        Self::null()
    }
}

impl<T: Object> Clone for Handle<T> {
    fn clone(&self) -> Self {
        Self {
            key: self.key.clone(),
            to_remove: self.to_remove.clone(),
            _phantom: PhantomData::default(),
        }
    }
}

impl<T: Object> Handle<T> {
    // Check if a handle is valid
    pub fn is_null(&self) -> bool {
        self.key.is_null()
    }
    // Create a new invalid handle
    pub fn null() -> Self {
        Self {
            // TODO: Optimize this
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
    // Actual map functionality
    pub fn map<R, F: FnOnce(&Self) -> R>(&self, f: F) -> Option<R> {
        (!self.is_null()).then(|| f(self))
    } 
}

impl<T: Object> Drop for Handle<T> {
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
