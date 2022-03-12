use std::cell::RefCell;

use bitfield::Bitfield;

use crate::component::{registry, Component, ComponentQueryParameters, ComponentQuerySet};

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
        self.system.subsystems.push(SubSystem { cbitfield: params.cbitfield, all: Default::default(), delta: Default::default() });
        self
    }
    // Set the "Run System" event of this system
    pub fn event(mut self, evn: fn(&mut World, ComponentQuerySet)) -> Self {
        self.system.evn_run = Some(evn);
        self
    }
    // Build this system and add it to the ECS manager
    pub fn build(self) {
        self.set.add(self.system)
    }
}
