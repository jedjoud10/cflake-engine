use crate::Handle;
use slotmap::DefaultKey;
use std::{marker::PhantomData, rc::Rc};

use super::Trackers;

// A weak handle is just like a normal handle, although it does not require that its
// owned element stays alive for the duration of the handle
// This can be used to "reserve" spots for non-initialized values and initialize them afterwards or
// for faillible fetching of values
pub struct Weak<T: 'static> {
    pub(super) _phantom: PhantomData<T>,
    pub(super) trackers: Rc<Trackers>,
    pub(super) key: DefaultKey,
}

impl<T: 'static> PartialEq for Weak<T> {
    fn eq(&self, other: &Self) -> bool {
        self.key == other.key
    }
}

impl<T: 'static> Eq for Weak<T> {}

impl<T: 'static> Weak<T> {
    // Get the current reference count for this handle
    pub fn count(&self) -> u32 {
        *self.trackers.counters.borrow().get(self.key).unwrap()
    }

    // Check if the owned value is still alive in the corresponding storage
    pub fn valid(&self) -> bool {
        self.count() > 0
    }

    // Try to upcast this weak handle to a strong one if it's value is still alive
    pub fn upgrade(self) -> Option<Handle<T>> {
        self.valid().then(|| unsafe {
            self.increment_count();
            Handle {
                _phantom: PhantomData,
                trackers: self.trackers.clone(),
                key: self.key,
            }
        })
    }

    // Overwrite the current reference counted value directly
    pub unsafe fn set_count(&self, count: u32) {
        let mut borrowed = self.trackers.counters.borrow_mut();
        *borrowed.get_mut(self.key).unwrap() = count;
    }

    // This will manually incremememnt the underlying reference counter
    pub unsafe fn increment_count(&self) -> u32 {
        let value = self.count().saturating_add(1);
        self.set_count(value);
        value
    }

    // This will manually decrement the underlying reference counter
    pub unsafe fn decrement_count(&self) -> u32 {
        let value = self.count().saturating_sub(1);
        self.set_count(value);
        value
    }
}

impl<T: 'static> Clone for Weak<T> {
    fn clone(&self) -> Self {
        Self {
            trackers: self.trackers.clone(),
            key: self.key,
            _phantom: PhantomData,
        }
    }
}
