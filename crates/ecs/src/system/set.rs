use std::{cell::RefCell, rc::Rc};

use crate::event::EcsEventSet;

use super::{System, SystemBuilder};
use getset::Getters;

// Systems
pub type Systems = Rc<RefCell<Vec<System>>>;

// System set
#[derive(Getters)]
pub struct SystemSet {
    #[getset(get = "pub")]
    pub(crate) inner: Systems,
    pub(crate) allowed_to_build: bool,
}

impl Default for SystemSet {
    fn default() -> Self {
        Self {
            inner: Default::default(),
            allowed_to_build: true,
        }
    }
}

impl SystemSet {
    // Create a new system build
    pub fn builder<World>(&mut self, ecs_events: &mut EcsEventSet<World>) -> SystemBuilder<World> {
        SystemBuilder::<World>::new(self, ecs_events.events.clone())
    }
    // Add a system to our current systems
    pub(crate) fn add(&mut self, system: System) {
        let mut borrowed = self.inner.borrow_mut();
        borrowed.push(system)
    }
    // Sort the systems based on their ordering
    pub fn sort(&mut self) {
        let mut vec = self.inner.borrow_mut();
        vec.sort_by(|a, b| a.order.cmp(&b.order))
    }
}
