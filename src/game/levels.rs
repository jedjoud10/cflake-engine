use crate::engine::core::world::World;
use crate::engine::core::ecs::Entity;
use crate::engine::core::ecs::System;
use crate::game::components::transforms::*;

// Load the default level
pub fn load_default_level(world: &mut World) {
	let mut camera: Entity = Entity::default();
	camera.name = String::from("Default Camera");
	camera.link_component::<Position, Position>(world, Position::default());	
	camera.link_component::<Scale, Scale>(world, Scale::default());	
	camera.link_component::<Rotation, Rotation>(world, Rotation::default());	

	let mut system: System = System::default();
	system.link_component::<Position>(world);
	system.link_component::<Scale>(world);
	system.link_component::<Rotation>(world);

	world.add_system(system);
	world.add_entity(camera);
}
