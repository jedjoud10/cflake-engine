use crate::engine::core::defaults::components::RenderComponent;
use crate::engine::core::defaults::components::UpdatableComponent;
use crate::engine::core::ecs::System;
use crate::engine::core::ecs::SystemData;
use crate::engine::core::ecs::SystemType;
use crate::engine::core::world::World;
use crate::engine::core::ecs::Entity;
use crate::engine::core::defaults::*;
use crate::game::components::transforms::*;
use crate::gl;

// Load the default level
pub fn load_default_level(world: &mut World) {
	// Create a camera entity
	let mut camera: Entity = Entity::default();
	camera.name = String::from("Default Camera");
	world.component_manager.register_component::<Position>();
	world.component_manager.register_component::<Scale>();
	world.component_manager.register_component::<Rotation>();
	camera.link_component::<Position>(world, Position::default());	
	camera.link_component::<Scale>(world, Scale::default());	
	camera.link_component::<Rotation>(world, Rotation::default());	
	
	// Default render system
	let mut default_render_system = System {
		system_data: SystemData::default(),
	};
	default_render_system.system_data.link_component::<RenderComponent>(world);

	// Render the entitites
	default_render_system.system_data.entity_loop_event = |entity, world| {
		let render_component = entity.get_component::<RenderComponent>(world);
	};
	// When an entity gets added to the render system
	default_render_system.system_data.entity_added_event = |entity, world| {

	};
	// When an entity gets removed from the render system
	default_render_system.system_data.entity_removed_event = |entity, world| {

	};
	default_render_system.system_data.stype = SystemType::Render;
	default_render_system.system_data.link_component::<RenderComponent>(world);

	// Add the default systems
	world.add_system(Box::new(default_render_system));
	world.add_entity(Box::new(camera));
	world.remove_entity(0);
}
