use crate::engine::core::ecs::{
	entity::Entity,
	system::System,
	system_data::{SystemData, SystemEventData, SystemEventDataLite},
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
	fn setup_system(&mut self, data: &mut SystemEventData) {}

	// Called for each entity in the system
	fn fire_entity(&mut self, entity: &mut Entity, data: &mut SystemEventData) {}

	// Add additional entities related to this system
	fn additional_entities(&mut self, _data: &mut SystemEventData) -> Vec<Entity> {
		Vec::new()
	}

	// Turn this into "Any" so we can cast into child systems
	fn as_any(&self) -> &dyn std::any::Any {
		return self;
	}

	fn as_any_mut(&mut self) -> &mut dyn std::any::Any {
		return self;
	}
}
