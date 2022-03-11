use std::cell::RefCell;

use crate::{
    component::{registry, Component, ComponentQueryParameters},
    event::EventKey,
};

use super::{SubSystem, System, SystemSet};

// A system builder used to build multiple systems
pub struct SystemBuilder<'a, World> {
    set: &'a mut SystemSet<World>,
    system: System<World>,
}

impl<'a, World> SystemBuilder<'a, World> {
    // Create a new system builder
    pub(crate) fn new(set: &'a mut SystemSet<World>) -> Self {
        Self { set, system: System::default() }
    }
    // Add a subsystem with specific component query parameters
    pub fn query(mut self, params: ComponentQueryParameters) -> Self {
        self.system.subsystems.push(RefCell::new(SubSystem::new(params)));
        self
    }
    // Set the "Run System" event of this system
    pub fn with_run_event(mut self, evn: fn(&mut World, EventKey)) -> Self {
        self.system.evn_run = Some(evn);
        self
    }
    // Build this system and add it to the ECS manager
    pub fn build(self) {
        self.set.add(self.system)
    }
}
