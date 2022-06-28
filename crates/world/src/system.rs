use crate::{Events, World};

// Systems are collections of multiple events that we insert onto the world
// Systems can be added onto the current app using the insert_system method
// Systems cannot be inserted more than once, since we keep track of their type internally
pub trait System: 'static {
    // Consume the system type and insert the corresponding events
    fn insert(self, events: &mut Events);
}

// Implementations of system for fnonce closures and function pointers
impl<F> System for F
where
    F: FnOnce(&mut Events) + 'static,
{
    fn insert(self, events: &mut Events) {
        self(events)
    }
}
