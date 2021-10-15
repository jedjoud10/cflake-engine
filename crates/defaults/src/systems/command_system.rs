use systems::System;

use crate::engine::core::ecs::{
    component::FilteredLinkedComponents,
    system::System,
    system_data::{SystemData, SystemEventData},
};

#[derive(Default)]
pub struct TemplateSystem {
    pub system_data: SystemData,
}

impl System for TemplateSystem {
    
    // Setup the system
    fn setup_system(&mut self, _data: &mut SystemEventData) {}

    // Called for each entity in the system
    fn fire_entity(&mut self, _components: &FilteredLinkedComponents, _data: &mut SystemEventData) {}    
}
