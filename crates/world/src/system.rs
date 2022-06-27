use crate::{Events, World};

// Systems are collections of multiple events that we insert onto the world
// Systems can be added onto the current app using the insert_system method
// Systems cannot be inserted more than once, since we keep track of their type
pub trait System: 'static {
    fn insert(self, events: &mut Events);
}

impl<F> System for F
where
    F: FnOnce(&mut Events) + 'static,
{
    fn insert(self, events: &mut Events) {
        self(events)
    }
}
