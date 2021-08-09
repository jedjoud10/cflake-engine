use crate::engine::core::ecs::{
    entity::Entity,
    system::System,
    system_data::{FireData, SystemData},
};

#[derive(Default)]
pub struct TemplateSystem {
    pub system_data: SystemData,
}

impl System for TemplateSystem {
    // Wrappers around system data
    fn get_system_data(&self) -> &SystemData {
        return &self.system_data;
    }

    fn get_system_data_mut(&mut self) -> &mut SystemData {
        return &mut self.system_data;
    }

    // Setup the system
    fn setup_system(&mut self, data: &mut FireData) {
    }

    // Called for each entity in the system
    fn fire_entity(&mut self, entity: &mut Entity, data: &mut FireData) {
    }
}
