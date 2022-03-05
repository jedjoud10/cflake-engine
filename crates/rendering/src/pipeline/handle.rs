use std::rc::Rc;
use slotmap::Key;

use super::InnerPipelineCollection;

// A unique pipeline collection key
slotmap::new_key_type! {
    pub(crate) struct PipelineElemKey;
}

// A strong handle to a pipeline object. If there are 0 strong handles, the pipeline object will be deallocated (totally not stolen from Bevy)
pub struct Handle<T> {
    pub(crate) key: Rc<PipelineElemKey>,
    pub(crate) inner: Option<InnerPipelineCollection<T>>,
}

impl<T> Default for Handle<T> {
    fn default() -> Self {
        Self { 
            key: Rc::new(PipelineElemKey::null()),
            inner: None
        }
    }
}

impl<T> Clone for Handle<T> {
    fn clone(&self) -> Self {
        Self { inner: self.inner.clone(), key: self.key.clone() }
    }
}

impl<T> Drop for Handle<T> {
    // Remove the element if this is the last strong handle it has
    fn drop(&mut self) {
        if let Some(inner) = self.inner {
            let strong_count = Rc::strong_count(&self.key);
            if strong_count == 0 {
                let inner = inner.borrow_mut();
                inner.remove(*self.key).unwrap();
            }
        }
    }
}

