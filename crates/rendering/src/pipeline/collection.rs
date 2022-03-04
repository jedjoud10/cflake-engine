use std::rc::Rc;

use slotmap::SlotMap;

// A pipeline collection that contains multiple elements of the same type
// This can only be accessed on the main thread
pub struct PipelineCollection<T> {
    // The inner storage
    inner: Rc<SlotMap<PipelineElemKey, T>>
}

// A unique pipeline collection key
slotmap::new_key_type! {
    struct PipelineElemKey;
}