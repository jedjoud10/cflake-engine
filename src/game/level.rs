use crate::engine::core::defaults::components::*;
use crate::engine::core::defaults::transforms::*;
use crate::engine::core::ecs::System;
use crate::engine::core::ecs::SystemData;
use crate::engine::core::ecs::SystemType;
use crate::engine::core::world::World;
use crate::engine::core::ecs::Entity;
use crate::engine::core::defaults::*;
use crate::gl;
// Pre-register unused components
pub fn register_components(world: &mut World) {
	world.component_manager.register_component::<Position>();
	world.component_manager.register_component::<Rotation>();
}
// Load the systems
pub fn load_systems(world: &mut World) {
	// Default render system
	let mut rs = System {
		system_data: SystemData::default(),
	};
	rs.system_data.link_component::<RenderComponent>(world);

	// When the render system gets updated
	rs.system_data.loop_event = |world| {
		unsafe {
			// Clear the window
			gl::Clear(gl::COLOR_BUFFER_BIT);
		}
	};
	// Render the entitites
	rs.system_data.entity_loop_event = |entity, world| {
		let rc = entity.get_component::<RenderComponent>(world);
	};
	// When an entity gets added to the render system
	rs.system_data.entity_added_event = |entity, world| {
		let rc = entity.get_component::<RenderComponent>(world);
		unsafe {
			// Create the vao
			gl::GenVertexArrays(1, rc.vertex_vao);
			gl::BindVertexArray(*rc.vertex_vao);
		}
	};
	// When an entity gets removed from the render system
	rs.system_data.entity_removed_event = |entity, world| {
		let rc = entity.get_component::<RenderComponent>(world);
		unsafe {
			// Delete the vertex array
			gl::DeleteVertexArrays(1, rc.vertex_vao);
		}
	};
	rs.system_data.stype = SystemType::Render;
	rs.system_data.link_component::<RenderComponent>(world);
	world.add_system(Box::new(rs));
}
// Load the entities
pub fn load_entities(world: &mut World) {	
	// Create a camera entity
	let mut camera= Entity::default();	
	camera.name = String::from("Default Camera");	
	camera.link_component::<Position>(world, Position::default());	
	camera.link_component::<Rotation>(world, Rotation::default());	
	
	// Simple cube to render
	let mut cube = Entity::default();
	cube.name = String::from("Cube");
	cube.link_component::<RenderComponent>(world, RenderComponent::default());

	world.add_entity(Box::new(camera));
	world.add_entity(Box::new(cube));
}
