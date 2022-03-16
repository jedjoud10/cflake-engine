use std::{
    cell::{Cell, Ref, RefCell},
    rc::Rc,
};

use crate::component::ComponentQuerySet;

// System execution event
pub type Event<World> = fn(&mut World, ComponentQuerySet);

pub type RcEvents<World> = Rc<RefCell<Vec<Event<World>>>>;

// Event manager
pub struct EcsEventSet<World> {
    pub(crate) events: RcEvents<World>,
}

impl<World> Clone for EcsEventSet<World> {
    fn clone(&self) -> Self {
        Self { events: self.events.clone() }
    }
}

impl<World> EcsEventSet<World> {
    // Get events
    pub fn events(&self) -> Ref<[Event<World>]> {
        let slice = Ref::map(self.events.borrow(), |vec| vec.as_slice());
        slice
    }
}

impl<World> Default for EcsEventSet<World> {
    fn default() -> Self {
        Self { events: Default::default() }
    }
}
