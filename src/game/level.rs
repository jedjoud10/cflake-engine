use crate::engine::core::defaults::components::components::*;
extern crate nalgebra_glm as glm;
use crate::engine::core::ecs::SystemState;
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
	let mut rs = System::default();
	rs.system_data.link_component::<RenderComponent>(world);
	rs.system_data.name = String::from("Rendering system");	

	// When the render system gets updated
	unsafe { 
		gl::ClearColor(1.0, 1.0, 1.0, 1.0);
		let default_size = World::get_default_window_size();
		gl::Viewport(0, 0, default_size.0, default_size.1)
	}
	rs.system_data.loop_event = |world| {
		unsafe {
			// Clear the window
			gl::Clear(gl::COLOR_BUFFER_BIT | gl::DEPTH_BUFFER_BIT);
		}
	};
	// Render the entitites
	rs.system_data.entity_loop_event = |entity, world| {	
		let id = entity.entity_id;
		let mut shader: &mut Shader;
		let mut model_matrix: glm::Mat4;
		let mut view_project_matrix: glm::Mat4;
		// Get the projection * view matrix
		{
			let camera_entity = world.get_entity(world.default_camera_id).clone();
			let mut rotation: glm::Quat;
			{
				rotation = camera_entity.get_component::<Rotation>(world).rotation;
			}
			let camera_data = camera_entity.get_component::<Camera>(world);
			let position = camera_entity.get_component::<Position>(world);
			// Just a simple lookat test
			rotation = glm::quat_look_at(&glm::normalize(&-position.position), &glm::vec3(0.0, 1.0, 0.0));
			view_project_matrix = camera_data.view_matrix * camera_data.projection_matrix;

			// Update the entity internally
			*camera_entity.get_component_mut::<Rotation>(world).rotation = *rotation;
			*world.get_entity(id) = camera_entity;
		}
		// Render the entity
		{
			let mut name= String::new();
			{
				let rc = entity.get_component::<RenderComponent>(world);
				name = rc.shader_name.clone();
				model_matrix = rc.gpu_data.model_matrix.clone();
			}
			shader = world.shader_manager.get_shader(&name).unwrap();
		}
		// Get the model matrix
		// Use the shader, and update any uniforms
		shader.use_shader();
		let loc = shader.get_uniform_location(String::from("mvp_matrix"));
		
		// Calculate the mvp matrix		
		let mvp_matrix: glm::Mat4 = view_project_matrix * model_matrix;
		// Pass the MVP to the shader
		shader.set_matrix_44_uniform(loc, mvp_matrix);

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
		let rc = entity.get_component_mut::<RenderComponent>(world);
		// Use the default shader for this entity renderer
		// Make sure we create the OpenGL data for this entity's model
		rc.refresh_model();
	};
	// When an entity gets removed from the render system
	rs.system_data.entity_removed_event = |entity, world| {
		let rc = entity.get_component_mut::<RenderComponent>(world);
		rc.dispose_model();
	};
	rs.system_data.stype = SystemType::Render;
	rs.system_data.link_component::<RenderComponent>(world);
	world.add_system(rs);

	// Create the default camera system
	let mut cs = System::default();
	cs.system_data.name = String::from("Camera System");
	cs.system_data.link_component::<Camera>(world);
	cs.system_data.link_component::<Position>(world);
	cs.system_data.link_component::<Rotation>(world);

	cs.system_data.entity_loop_event = |entity, world| {
		let mut position: glm::Vec3;
		let mut rotation: glm::Quat;
		{
			// Set the variables since we can't have two mutable references at once
			rotation = entity.get_component::<Rotation>(world).rotation;
			position = entity.get_component::<Position>(world).position;
		}
		let mut camera_data = entity.get_component_mut::<Camera>(world);
		// Update the view matrix every time we make a change
		camera_data.update_view_matrix(&position, &rotation);
	};
	//cs.system_data.state = SystemState::Disabled(0.0);
	world.add_system(cs);
}
// Load the entities
pub fn load_entities(world: &mut World) {	
	// Create a camera entity
	let mut camera= Entity::default();	
	camera.name = String::from("Default Camera");	
	camera.link_component::<Position>(world, Position::default());	
	camera.link_component::<Rotation>(world, Rotation::default());	
	camera.link_component::<Camera>(world, Camera::default());

	// Make it the default camera
	world.default_camera_id = world.add_entity(camera);
	
	// Load the default shader
	let mut default_shader = Shader::default();
	{
		{
			let default_frag_subshader_resource = world.resource_manager.load_resource(String::from("default.frsh.glsl.pkg"), String::from("shaders\\")).unwrap();
			// Link the vertex and fragment shaders
			let mut frag_subshader = world.shader_manager.create_subshader_from_resource(default_frag_subshader_resource).unwrap();
			// Compile the subshader
			frag_subshader.compile_subshader();
			// Cache it, and link it
			let mut frag_subshader = world.shader_manager.cache_subshader(frag_subshader).unwrap();
			default_shader.link_subshader(&frag_subshader);
		}
		{
			let default_vert_subshader_resource = world.resource_manager.load_resource(String::from("default.vrsh.glsl.pkg"), String::from("shaders\\")).unwrap();
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
		vertices: vec![glm::Vec3::new(-1.0, -1.0, 0.0), glm::Vec3::new(1.0, -1.0, 0.0), glm::Vec3::new(0.0, 1.0, 0.0)],
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
	world.add_entity(cube);
}
