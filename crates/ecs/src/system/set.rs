use std::{rc::Rc, cell::RefCell};

use super::{System, SystemBuilder};
use getset::Getters;

// Systems
pub type Systems<World> = Rc<RefCell<Vec<System<World>>>>;

// System set
#[derive(Getters)]
pub struct SystemSet<World> {
    #[getset(get = "pub")]
    pub(crate) inner: Systems<World>,
}

impl<World> Default for SystemSet<World> {
    fn default() -> Self {
        Self { inner: Default::default() }
    }
}

impl<World> SystemSet<World> {
    // Create a new system build
    pub fn builder(&mut self) -> SystemBuilder<World> {
        SystemBuilder::new(self)
    }
    // Add a system to our current systems
    pub(crate) fn add(&mut self, system: System<World>) {
        let mut borrowed = self.inner.borrow_mut();
        borrowed.push(system)
    }
}