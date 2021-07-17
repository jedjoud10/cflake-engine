use crate::engine::core::world::World;
use crate::engine::core::ecs::Entity;
use crate::game::components::transforms::*;

// Load the default level
pub fn load_default_level(world: &mut World) {
	let mut camera: Entity = Entity::default();
	camera.name = String::from("Default Camera");
	camera.link_component::<Position, Position>(world, Position::default());	
	world.add_entity(camera);
}
