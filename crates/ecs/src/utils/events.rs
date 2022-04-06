use std::{
    cell::RefCell,
    rc::Rc,
};

use crate::EventExecutionOrder;

// A single event
pub type Event<World> = fn(&mut World);

// Multiple events that will be stored in the world
pub struct SystemSet<World> {
    pub(crate) inner: Rc<RefCell<Vec<(i32, Event<World>)>>>,
}

impl<World> SystemSet<World> {
    // Insert an event into the system set
    pub fn insert(&mut self, evn: fn(&mut World)) {
        let idx = EventExecutionOrder::fetch_add();
        self.inner.borrow_mut().push((idx, evn));
    }
    // Insert an event that executes at a specific order index
    pub fn insert_with(&mut self, evn: fn(&mut World), order: i32) {
        self.inner.borrow_mut().push((order, evn));
    }
    // Sort the systems based on their execution order index
    pub fn sort(&mut self) {
        let mut borrowed = self.inner.borrow_mut();
        borrowed.sort_by(|(a, _), (b, _)| i32::cmp(a, b));
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
