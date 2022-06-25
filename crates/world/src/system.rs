use crate::{Events, World};

// Systems are collections of multiple events that we insert onto the world
// Systems can be added onto the current app using the insert_system method
pub trait System {
    // Consuime the system and insert the events
    fn insert(self, events: &Events);
}
