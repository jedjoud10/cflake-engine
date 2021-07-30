use glam::Vec4Swizzles;
use crate::engine::core::defaults::components::{components, *};
use crate::engine::core::ecs::{SystemType, SystemData, SystemState, System, Entity};
use crate::engine::core::world::World;

// Create the skysphere system
pub fn create_system(world: &mut World) {
	let mut system = System::default();
	system.system_data.name = String::from("Skybox System");
	system.system_data.link_component::<components::Skysphere>(world);
	system.system_data.link_component::<transforms::Position>(world);
	system.system_data.link_component::<transforms::Scale>(world);
	system.system_data.entity_loop_event = |entity, world| {
		// Set the position of the sky sphere to always be the camera
		let position = world.get_entity(world.default_camera_id).get_component::<transforms::Position>(world).position.clone();
		*entity.get_component_mut::<transforms::Position>(world).position = *position;
	};

	world.add_system(system);
}