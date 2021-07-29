use std::ffi::CString;
use glam::Vec4Swizzles;

use crate::engine::core::defaults::components::{components, *};
use crate::engine::rendering::*;
use crate::engine::core::ecs::{SystemType, SystemData, SystemState, System, Entity};
use crate::engine::core::world::World;
use crate::gl;
// Pre-register unused components
pub fn register_components(world: &mut World) {
	world.component_manager.register_component::<transforms::Position>();
	world.component_manager.register_component::<transforms::Rotation>();
}
// Load the systems
pub fn load_systems(world: &mut World) {
	// Default render system
	let mut rs = System::default();
	rs.system_data.name = String::from("Rendering system");	
	rs.system_data.link_component::<components::Render>(world);
	rs.system_data.link_component::<transforms::Position>(world);
	rs.system_data.link_component::<transforms::Position>(world);

	// When the render system gets updated
	unsafe { 
		gl::ClearColor(0.0, 0.0, 0.0, 0.0);
		let default_size = World::get_default_window_size();
		gl::Viewport(0, 0, default_size.0, default_size.1);
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
		let mut view_matrix: glam::Mat4;
		let mut projection_matrix: glam::Mat4;
		// Get the projection * view matrix
		{
			let camera_entity = world.get_entity(world.default_camera_id);
			let camera_data = camera_entity.get_component::<components::Camera>(world);
			projection_matrix = camera_data.projection_matrix;
			view_matrix = camera_data.view_matrix;
		}
		let mut model_matrix: glam::Mat4;
		// Render the entity
		{
			let mut name= String::new();
			// Get the model matrix
			{
				let position: glam::Vec3;
				let rotation: glam::Quat;
				{
					position = entity.get_component::<transforms::Position>(world).position;
					rotation = entity.get_component::<transforms::Rotation>(world).rotation;
				}
				let rc = entity.get_component_mut::<components::Render>(world);
				rc.update_model_matrix(position.clone(), rotation.clone());
				name = rc.shader_name.clone();
				model_matrix = rc.gpu_data.model_matrix.clone();
			}
			shader = world.shader_manager.get_shader(&name).unwrap();
		}
		// Use the shader, and update any uniforms
		shader.use_shader();
		
		let mut loc = shader.get_uniform_location(CString::new("mvp_matrix").unwrap());
		// Calculate the mvp matrix		
		let mvp_matrix: glam::Mat4 = projection_matrix * view_matrix * model_matrix;
		// Pass the MVP to the shader
		shader.set_matrix_44_uniform(loc, mvp_matrix);

		unsafe {
			// Actually draw the array
			let rc = entity.get_component::<components::Render>(world);
			if rc.gpu_data.initialized {
				gl::BindBuffer(gl::ARRAY_BUFFER, rc.gpu_data.vertex_buf);
				gl::DrawArrays(gl::TRIANGLES, 0, 3);
			}
		}
	};
	// When an entity gets added to the render system
	rs.system_data.entity_added_event = |entity, world| {
		let rc = entity.get_component_mut::<components::Render>(world);
		// Use the default shader for this entity renderer
		// Make sure we create the OpenGL data for this entity's model
		rc.refresh_model();
	};
	// When an entity gets removed from the render system
	rs.system_data.entity_removed_event = |entity, world| {
		let rc = entity.get_component_mut::<components::Render>(world);
		rc.dispose_model();
	};
	rs.system_data.stype = SystemType::Render;
	rs.system_data.link_component::<components::Render>(world);
	world.add_system(rs);

	// Create the default camera system
	let mut cs = System::default();
	cs.system_data.name = String::from("Camera System");
	cs.system_data.link_component::<components::Camera>(world);
	cs.system_data.link_component::<transforms::Position>(world);
	cs.system_data.link_component::<transforms::Rotation>(world);

	cs.system_data.entity_added_event = |entity, world| {
		// First time we initialize the camera, setup the matrices
		let mut position: glam::Vec3;
		let mut rotation: glam::Quat;
		{
			// Set the variables since we can't have two mutable references at once
			rotation = entity.get_component::<transforms::Rotation>(world).rotation;
			position = entity.get_component::<transforms::Position>(world).position;
		}
		let mut camera_component = entity.get_component_mut::<components::Camera>(world);
		camera_component.update_projection_matrix();
		camera_component.update_view_matrix(position, rotation);
		world.input_manager.bind_key(glfw::Key::W, String::from("camera_forward"));
		world.input_manager.bind_key(glfw::Key::S, String::from("camera_backwards"));
		world.input_manager.bind_key(glfw::Key::D, String::from("camera_right"));
		world.input_manager.bind_key(glfw::Key::A, String::from("camera_left"));
		world.input_manager.bind_key(glfw::Key::Space, String::from("camera_up"));
		world.input_manager.bind_key(glfw::Key::LeftShift, String::from("camera_down"));
	};

	cs.system_data.entity_loop_event = |entity, world| {
		let mut position: glam::Vec3;
		let mut rotation: glam::Quat;
		{
			// Create some movement using user input
			{
				let delta_time = world.time_manager.delta_time as f32;
				let mut changed_rotation = entity.get_component_mut::<transforms::Rotation>(world).rotation.clone();

				// Rotate the camera around
				let mouse_pos = world.input_manager.get_accumulated_mouse_position();
				let sensitivity = 0.001_f32;
				changed_rotation = glam::Quat::from_euler(glam::EulerRot::XYZ, -mouse_pos.1 as f32 * sensitivity, -mouse_pos.0 as f32 * sensitivity, 0.0);
			
				// Keyboard input
				let forward_vector = glam::Mat4::from_quat(changed_rotation).mul_vec4(glam::vec4(0.0, 0.0, 1.0, 1.0)).xyz();
				let up_vector = glam::Mat4::from_quat(changed_rotation).mul_vec4(glam::vec4(0.0, 1.0, 0.0, 1.0)).xyz();
				let right_vector = glam::Mat4::from_quat(changed_rotation).mul_vec4(glam::vec4(1.0, 0.0, 0.0, 1.0)).xyz();
				let changed_position = &mut entity.get_component::<transforms::Position>(world).position.clone();
				if world.input_manager.map_held(String::from("camera_forward")).0 {
					*changed_position += -forward_vector * world.time_manager.delta_time as f32;
				} else if world.input_manager.map_held(String::from("camera_backwards")).0 {
					*changed_position += forward_vector * world.time_manager.delta_time as f32;
				}
				if world.input_manager.map_held(String::from("camera_right")).0 {
					*changed_position += right_vector * world.time_manager.delta_time as f32;
				} else if world.input_manager.map_held(String::from("camera_left")).0 {
					*changed_position += -right_vector * world.time_manager.delta_time as f32;
				}
				if world.input_manager.map_held(String::from("camera_up")).0 {
					*changed_position += up_vector * world.time_manager.delta_time as f32;
				} else if world.input_manager.map_held(String::from("camera_down")).0 {
					*changed_position += -up_vector * world.time_manager.delta_time as f32;
				}
				// Update the variables
				*entity.get_component_mut::<transforms::Position>(world).position = **changed_position;
				entity.get_component_mut::<transforms::Rotation>(world).rotation = changed_rotation;
				position = *changed_position;
				rotation = changed_rotation.clone();
			}
		}
		let mut camera_component = entity.get_component_mut::<components::Camera>(world);
		// Update the view matrix every time we make a change
		camera_component.update_view_matrix(position, rotation);
	};

	world.add_system(cs);
}
// Load the entities
pub fn load_entities(world: &mut World) {	
	// Create a camera entity
	let mut camera= Entity::default();	
	camera.name = String::from("Default Camera");	
	camera.link_component::<transforms::Position>(world, transforms::Position {
		position: glam::vec3(5.0, 5.0, 5.0),
	});	
	camera.link_component::<transforms::Rotation>(world, transforms::Rotation::default());	
	camera.link_component::<components::Camera>(world, components::Camera::default());

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
		vertices: vec![glam::vec3(-1.0, -1.0, 0.0), glam::vec3(1.0, -1.0, 0.0), glam::vec3(0.0, 1.0, 0.0)],
		triangles: vec![0, 1, 2],
	};
	// Link the component
	let rc = components::Render {
    	render_state: EntityRenderState::Visible,
    	gpu_data: ModelDataGPU::default(),
    	shader_name: default_shader_name.clone(),   
		model	
	};
	cube.link_component::<components::Render>(world, rc);
	cube.link_component::<transforms::Position>(world, transforms::Position::default());
	cube.link_component::<transforms::Rotation>(world, transforms::Rotation::default());
	world.add_entity(cube);
	// Create another cube
	let mut cube = Entity::default();
	cube.name = String::from("Cube 2");
	// Create the model
	let model = Model {
		vertices: vec![glam::vec3(0.0, -1.0, -1.0), glam::vec3(0.0, 1.0, -1.0), glam::vec3(0.0, 0.0, 1.0)],
		triangles: vec![0, 1, 2],
	};
	// Link the component
	let mut rc = components::Render {
    	render_state: EntityRenderState::Visible,
    	gpu_data: ModelDataGPU::default(),
    	shader_name: default_shader_name.clone(),   
		model	
	};
	cube.link_component::<components::Render>(world, rc);
	cube.link_component::<transforms::Position>(world, transforms::Position {
		position: glam::vec3(5.0, 0.0, 0.0),
	});
	cube.link_component::<transforms::Rotation>(world, transforms::Rotation::default());
	//world.add_entity(cube);
	
}
