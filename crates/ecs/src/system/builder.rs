use bitfield::Bitfield;

use crate::{ECSManager, component::{Component, registry, ComponentQuery}};

use super::{System, EventHandler};

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
    // Set the run event of this system
    pub fn set_event(mut self, evn: fn(Context, ComponentQuery)) -> Self {
        self.system.run_event_idx = self.ecs_manager.event_handler.add_run_event(evn) as isize;
        self
    }
    // Build this system and add it to the ECS manager
    pub fn build(self) {
        self.ecs_manager.add_system(self.system)
    }
}