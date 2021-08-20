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
    // Wrappers around system data
    fn get_system_data(&self) -> &SystemData {
        &self.system_data
    }

    fn get_system_data_mut(&mut self) -> &mut SystemData {
        &mut self.system_data
    }

    // Setup the system
    fn setup_system(&mut self, _data: &mut SystemEventData) {}

    // Called for each entity in the system
    fn fire_entity(
        &mut self,
        _components: &mut FilteredLinkedComponents,
        _data: &mut SystemEventData,
    ) {
    }

    // Turn this into "Any" so we can cast into child systems
    fn as_any(&self) -> &dyn std::any::Any {
        self
    }

    fn as_any_mut(&mut self) -> &mut dyn std::any::Any {
        self
    }
}
