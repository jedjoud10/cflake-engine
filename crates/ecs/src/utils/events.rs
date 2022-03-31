use std::{cell::RefCell, rc::Rc};

// A single event
pub type Event<World> = fn(&mut World);

// Multiple events that will be stored in the world
pub struct SystemSet<World> {
    pub(crate) inner: Rc<RefCell<Vec<Event<World>>>>,
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
