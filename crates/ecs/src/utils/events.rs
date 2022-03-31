use std::{cell::RefCell, rc::Rc};

// A single event
pub type Event<World> = fn(&mut World);

// Multiple events that will be stored in the world
pub struct SystemSet<World> {
    pub(crate) inner: Rc<RefCell<Vec<Event<World>>>>,
}

impl<World> SystemSet<World> {
    // Insert an event into the system set
    pub fn insert(&mut self, evn: fn(&mut World)) {
        self.inner.borrow_mut().push(evn);
    }
}

impl<World> Default for SystemSet<World> {
    fn default() -> Self {
        Self { inner: Default::default() }
    }
}

impl<World> Clone for SystemSet<World> {
    fn clone(&self) -> Self {
        Self { inner: self.inner.clone() }
    }
}
