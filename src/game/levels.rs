use crate::engine::core::world::World;
use crate::engine::core::ecs::Entity;
use crate::game::components::transforms::*;

// Load the default level
pub fn load_default_level(world: &mut World) {
	let mut camera: Entity = Entity::default();
	let mut camera_position: Position;
	camera.name = String::from("Default Camera");
	camera.add_component(world.component_manager.get_component_id<Position>());
	world.add_entity(camera);
}
