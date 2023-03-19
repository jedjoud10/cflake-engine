use super::Trackers;
use slotmap::DefaultKey;
use std::{marker::PhantomData, rc::Rc, sync::{Arc, atomic::Ordering}};

// A handle is what keeps the values within Storage<T> alive
// Fetching data using this type of Handle is always successful
pub struct Handle<T: 'static> {
    pub(super) _phantom: PhantomData<T>,
    pub(super) trackers: Arc<Trackers>,
    pub(super) key: DefaultKey,
}

impl<T: 'static> PartialEq for Handle<T> {
    fn eq(&self, other: &Self) -> bool {
        self.key == other.key
    }
}

impl<T: 'static> Eq for Handle<T> {}

impl<T: 'static> Handle<T> {
    // Get the current reference count for this handle
    pub fn count(&self) -> u32 {
        self.trackers.counters.read().get(self.key).unwrap().load(Ordering::Relaxed)
    }

    // Overwrite the current reference counted value directly
    pub unsafe fn set_count(&self, count: u32) {
        let borrowed = self.trackers.counters.read();
        borrowed.get(self.key).unwrap().store(count, Ordering::Relaxed);
    }

    // This will manually incremememnt the underlying reference counter
    pub unsafe fn increment_count(&self) -> u32 {
        let borrowed = self.trackers.counters.read();
        borrowed.get(self.key).unwrap().fetch_add(1, Ordering::Relaxed)
    }

    // This will manually decrement the underlying reference counter
    pub unsafe fn decrement_count(&self) -> u32 {
        let borrowed = self.trackers.counters.read();
        borrowed.get(self.key).unwrap().fetch_sub(1, Ordering::Relaxed)
    }
}

// Cloning the handle will increase the reference count of that handle
impl<T: 'static> Clone for Handle<T> {
    fn clone(&self) -> Self {
        unsafe {
            self.increment_count();
        }

        Self {
            trackers: self.trackers.clone(),
            key: self.key,
            _phantom: PhantomData,
        }
    }
}

// Dropping the handle will decrease the reference count of that handle
// If we drop the last valid handle, then the stored value will get dropped
impl<T: 'static> Drop for Handle<T> {
    fn drop(&mut self) {
        // If the counter reaches 0, it means that we must drop the inner value
        if unsafe { self.decrement_count() } == 0 {
            self.trackers.dropped.lock().push(self.key);
            self.trackers.cleaned.store(false, Ordering::Relaxed);
        }
    }
}
