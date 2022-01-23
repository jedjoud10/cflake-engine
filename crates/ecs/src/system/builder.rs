use crate::{
    component::{registry, Component, ComponentQuery},
    ECSManager, utils::GlobalComponentError,
};

use super::System;

// A system builder used to build multiple systems
pub struct SystemBuilder<'a, Context> {
    ecs_manager: &'a mut ECSManager<Context>,
    system: System,
}

impl<'a, Context> SystemBuilder<'a, Context> {
    // Create a new system builder
    pub fn new(ecs_manager: &'a mut ECSManager<Context>) -> Self {
        Self {
            ecs_manager,
            system: System::default(),
        }
    }
    // Link a component to this system
    pub fn link<U: Component>(mut self) -> Self {
        let c = registry::get_component_bitfield::<U>();
        self.system.cbitfield = self.system.cbitfield.add(&c);
        self
    }
    // Set the "Run System" event of this system
    pub fn set_run_event(mut self, evn: fn(Context, ComponentQuery)) -> Self {
        self.system.evn_run = Some(self.ecs_manager.event_handler.add_run_event(evn));
        self
    }
    // Set the "Added Entity" event of this system
    pub fn set_added_entities_event(mut self, evn: fn(Context, ComponentQuery)) -> Self {
        self.system.evn_added_entity = Some(self.ecs_manager.event_handler.add_added_entity_event(evn));
        self
    }
    // Set the "Removed Entity" event of this system
    pub fn set_removed_entities_event(mut self, evn: fn(Context, ComponentQuery)) -> Self {
        self.system.evn_removed_entity = Some(self.ecs_manager.event_handler.add_removed_entity_event(evn));
        self
    }
    // Build this system and add it to the ECS manager
    pub fn build(self) {
        self.ecs_manager.add_system(self.system)
    }
}
