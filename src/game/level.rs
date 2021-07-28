use nalgebra::Point3;

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

	// When the render system gets updated
	unsafe { 
		gl::ClearColor(1.0, 1.0, 1.0, 1.0);
	}
	rs.system_data.loop_event = |world| {
		unsafe {
			// Clear the window
			gl::Clear(gl::COLOR_BUFFER_BIT | gl::DEPTH_BUFFER_BIT);
		}
	};
	// Render the entitites
	rs.system_data.entity_loop_event = |entity, world| {	
		let mut shader: &mut Shader;
		// Render the entity
		{
			let mut name= String::new();
			{
				let rc = entity.get_component::<RenderComponent>(world);
				name = rc.shader_name.clone();
			}
			shader = world.shader_manager.get_shader(&name).unwrap();
		}
		// Use the shader, and update any uniforms
		shader.use_shader();
		let loc = shader.get_uniform_location(String::from("test"));
		shader.set_scalar_1_uniform(loc, world.time_manager.time_since_start.sin() as f32);
		
		unsafe {
			// Actually draw the array
			let rc = entity.get_component::<RenderComponent>(world);
			if rc.gpu_data.initialized {
				gl::BindBuffer(gl::ARRAY_BUFFER, rc.gpu_data.vertex_buf);
				gl::DrawArrays(gl::TRIANGLES, 0, 3);
			}
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
	
	// Load the default shader
	let mut default_shader = Shader::default();
	{
		{
			let default_frag_subshader_resource = world.resource_manager.load_resource(String::from("default.frsh.pkg"), String::from("shaders\\")).unwrap();
			// Link the vertex and fragment shaders
			let mut frag_subshader = world.shader_manager.create_subshader_from_resource(default_frag_subshader_resource).unwrap();
			// Compile the subshader
			frag_subshader.compile_subshader();
			// Cache it, and link it
			let mut frag_subshader = world.shader_manager.cache_subshader(frag_subshader).unwrap();
			default_shader.link_subshader(&frag_subshader);
		}
		{
			let default_vert_subshader_resource = world.resource_manager.load_resource(String::from("default.vrsh.pkg"), String::from("shaders\\")).unwrap();
			// Link the vertex and fragment shaders
			let mut vert_subshader = world.shader_manager.create_subshader_from_resource(default_vert_subshader_resource).unwrap();
			// Compile the subshader
			vert_subshader.compile_subshader();
			// Cache it, and link it
			let mut vert_subshader = world.shader_manager.cache_subshader(vert_subshader).unwrap();
			default_shader.link_subshader(&vert_subshader);
		}
	}	
	let default_shader_name = default_shader.name.clone();
	let mut default_shader = world.shader_manager.cache_shader(default_shader).unwrap();
	// Use it for the default rendering of everything
	default_shader.use_shader();

	// Simple cube to render
	let mut cube = Entity::default();
	cube.name = String::from("Cube");
	// Create the model
	let model = Model {
    	vertices: vec![Point3::new(-1.0, -1.0, 0.0), Point3::new(1.0, -1.0, 0.0), Point3::new(0.0, 1.0, 0.0)],
    	triangles: vec![0, 1, 2],
	};
	// Link the component
	let rc = RenderComponent {
    	render_state: EntityRenderState::Visible,
    	gpu_data: ModelDataGPU::default(),
    	shader_name: default_shader_name,   
		model	
	};
	cube.link_component::<RenderComponent>(world, rc);

	world.add_entity(camera);
	world.add_entity(cube);
}
