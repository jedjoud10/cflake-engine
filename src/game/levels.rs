use crate::engine::core::world::World;
use crate::engine::core::ecs::Entity;
use crate::engine::core::ecs::System;
use crate::game::components::transforms::*;
use crate::gl;

// Load the default level
pub fn load_default_level(world: &mut World) {
	// Create a camera entity
	let mut camera: Entity = Entity::default();
	camera.name = String::from("Default Camera");
	camera.link_component::<Position, Position>(world, Position::default());	
	camera.link_component::<Scale, Scale>(world, Scale::default());	
	camera.link_component::<Rotation, Rotation>(world, Rotation::default());	

	
	// Add the systems first, then the entities
	world.add_entity(Box::new(camera));
}
