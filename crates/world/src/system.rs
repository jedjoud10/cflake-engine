use crate::{Events, World};

// Systems are collections of multiple events that we insert onto the world
// Systems can be added onto the current app using the insert_system method
pub trait System {
    // Consume the system and insert the events
    fn insert(self, events: &Events);
}

impl<F> System for F
where
    F: FnOnce(&Events) + 'static,
{
    fn insert(self, events: &Events) {
        self(events)
    }
}
