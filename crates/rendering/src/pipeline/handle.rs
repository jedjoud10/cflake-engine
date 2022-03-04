use std::rc::Rc;
use super::InnerPipelineCollection;

// A unique pipeline collection key
slotmap::new_key_type! {
    pub(crate) struct PipelineElemKey;
}

// A strong handle to a pipeline object. If there are 0 strong handles, the pipeline object will be deallocated (totally not stolen from Bevy)
pub struct Handle<T> {
    pub(crate) inner: InnerPipelineCollection<T>,
    pub(crate) key: Rc<PipelineElemKey>,
}

impl<T> Drop for Handle<T> {
    // Remove the element if this is the last strong handle it has
    fn drop(&mut self) {
        let strong_count = Rc::strong_count(&self.key);
        if strong_count == 0 {
            let inner = self.inner.borrow_mut();
            inner.remove(self.key).unwrap();
        }
    }
}

