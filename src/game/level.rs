use crate::engine::core::defaults::components::components::*;
use crate::engine::rendering::*;
use crate::engine::core::defaults::components::transforms::*;
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
	rs.system_data.name = String::from("Rendering system");
	// Load the default shader
	//let mut default_shader = Shader::default();
	{
		{
			let default_frag_subshader_resource = world.resource_manager.load_resource(String::from("default.frsh.pkg"), String::from("shaders\\")).unwrap();
			// Link the vertex and fragment shaders
			let mut frag_subshader = world.shader_manager.create_subshader_from_resource(default_frag_subshader_resource).unwrap();
			// Compile the subshader
			frag_subshader.compile_subshader();
			// Then cache it
			world.shader_manager.cache_subshader(&frag_subshader, String::from("default.frsh.pkg"));
			// Then read from the shader cache
			//default_shader.link_subshader(&frag_subshader);
		}
		{
			let default_vert_subshader_resource = world.resource_manager.load_resource(String::from("default.vrsh.pkg"), String::from("shaders\\")).unwrap();
			// Link the vertex and fragment shaders
			let mut vert_subshader = world.shader_manager.create_subshader_from_resource(default_vert_subshader_resource).unwrap();
			// Compile the subshader
			vert_subshader.compile_subshader();
			// Then cache it
			world.shader_manager.cache_subshader(&vert_subshader, String::from("default.vrsh.pkg"));
			// Then read from the shader cache
			//default_shader.link_subshader(&vert_subshader);
		}
	}	

	// Use it for the default rendering of everything
	//default_shader.use_shader();

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
		unsafe {
			// Render the entity
		}
	};
	// When an entity gets added to the render system
	rs.system_data.entity_added_event = |entity, world| {
		let rc = entity.get_component::<RenderComponent>(world);
		// Use the default shader for this entity renderer
		// Make sure we create the OpenGL data for this entity's model
		rc.refresh_model();
	};
	// When an entity gets removed from the render system
	rs.system_data.entity_removed_event = |entity, world| {
		let rc = entity.get_component::<RenderComponent>(world);
		rc.dispose_model();
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
