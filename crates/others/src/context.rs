use std::sync::{Arc, RwLock, RwLockReadGuard};

// Some context that we can save so we read it later
// This is a pointer that we get on the main thread, and pass it to the other threads
// The value T will never be written to during a frame, so we save performance and use a pointer directly instead of an Arc<RwLock<T>>
pub struct Context<T> {
    val: *const T,
}

impl<T> std::ops::Deref for Context<T> {
    type Target = T;

    // Nobody will be writing to the context during we read it, so this is totally fine
    fn deref(&self) -> &Self::Target {
        let ptr = unsafe { &*self.val };
        ptr
    }
}
