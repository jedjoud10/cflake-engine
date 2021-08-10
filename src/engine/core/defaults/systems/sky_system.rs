use crate::engine::core::defaults::components::{components, *};
use crate::engine::core::world::World;
use glam::Vec4Swizzles;

use crate::engine::core::ecs::{
	entity::Entity,
	system::System,
	system_data::{FireData, SystemData},
};

#[derive(Default)]
pub struct SkySystem {
	pub system_data: SystemData,
}

impl System for SkySystem {
	// Wrappers around system data
	fn get_system_data(&self) -> &SystemData {
		return &self.system_data;
	}

	fn get_system_data_mut(&mut self) -> &mut SystemData {
		return &mut self.system_data;
	}

	// Setup the system
	fn setup_system(&mut self, data: &mut FireData) {
		let system_data = self.get_system_data_mut();
		system_data.link_component::<components::Sky>(data.component_manager);
		system_data.link_component::<transforms::Position>(data.component_manager);
		system_data.link_component::<transforms::Scale>(data.component_manager);
	}

	// Called for each entity in the system
	fn fire_entity(&mut self, entity: &mut Entity, data: &mut FireData) {
		// Set the position of the sky sphere to always be the camera
		let position = data
			.entity_manager
			.get_entity(data.custom_data.main_camera_entity_id)
			.get_component::<transforms::Position>(data.component_manager)
			.position;
		*entity
			.get_component_mut::<transforms::Position>(data.component_manager)
			.position = *position;
	}

	// Turn this into "Any" so we can cast into child systems
	fn as_any(&self) -> &dyn std::any::Any {
		return self;
	}

	fn as_any_mut(&mut self) -> &mut dyn std::any::Any {
		return self;
	}
}
