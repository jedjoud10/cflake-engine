use super::InnerPipelineCollection;
use bitfield::AtomicSparseBitfield;
use parking_lot::Mutex;
use slotmap::{Key, KeyData};
use std::{marker::PhantomData, rc::Rc, sync::Arc};

// A unique pipeline collection key
slotmap::new_key_type! {
    pub(crate) struct PipelineElemKey;
}

// A strong handle to a pipeline object. If there are 0 strong handles, the pipeline object will be deallocated (totally not stolen from Bevy)
pub struct Handle<T> {
    pub(crate) key: Arc<PipelineElemKey>,
    pub(crate) to_remove: Option<Arc<Mutex<Vec<PipelineElemKey>>>>,
    pub(crate) _phantom: PhantomData<T>,
}

impl<T> Default for Handle<T> {
    fn default() -> Self {
        Self {
            key: Arc::new(PipelineElemKey::null()),
            to_remove: None,
            _phantom: PhantomData::default(),
        }
    }
}

impl<T> Clone for Handle<T> {
    fn clone(&self) -> Self {
        Self {
            key: self.key.clone(),
            to_remove: self.to_remove.clone(),
            _phantom: PhantomData::default(),
        }
    }
}

impl<T> Drop for Handle<T> {
    // Remove the element if this is the last strong handle it has
    fn drop(&mut self) {
        if let Some(to_remove) = &self.to_remove {
            let strong_count = Arc::strong_count(&self.key);
            if strong_count == 0 {
                // Remove the element that this Handle referred to
                let mut inner = to_remove.lock();
                inner.push(self.key.as_ref().clone());
            }
        }
    }
}
