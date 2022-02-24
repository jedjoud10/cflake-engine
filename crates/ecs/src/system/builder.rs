use crate::{
    component::{registry, Component},
    event::EventKey,
    ECSManager,
};

use super::System;

// A system builder used to build multiple systems
pub struct SystemBuilder<'a, World> {
    ecs_manager: &'a mut ECSManager<World>,
    system: System<World>,
}

impl<'a, World> SystemBuilder<'a, World> {
    // Create a new system builder
    pub(crate) fn new(ecs_manager: &'a mut ECSManager<World>) -> Self {
        Self {
            ecs_manager,
            system: System::default(),
        }
    }
    // Link a component to this system
    pub fn link<U: Component + 'static>(mut self) -> Self {
        let c = registry::get_component_bitfield::<U>();
        self.system.cbitfield = self.system.cbitfield.add(&c);
        self
    }
    // Set the "Run System" event of this system
    pub fn with_run_event(mut self, evn: fn(&mut World, EventKey)) -> Self {
        self.system.evn_run = Some(evn);
        self
    }
    // Set the "Added Entity" event of this system
    pub fn with_added_entities_event(mut self, evn: fn(&mut World, EventKey)) -> Self {
        self.system.evn_added_entity = Some(evn);
        self
    }
    // Set the "Removed Entity" event of this system
    pub fn with_removed_entities_event(mut self, evn: fn(&mut World, EventKey)) -> Self {
        self.system.evn_removed_entity = Some(evn);
        self
    }
    // Build this system and add it to the ECS manager
    pub fn build(self) {
        self.ecs_manager.add_system(self.system)
    }
}
