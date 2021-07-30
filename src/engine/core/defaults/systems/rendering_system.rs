use std::ffi::CString;
use std::ptr::null;
use crate::engine::rendering::*;
use crate::engine::core::defaults::components::{components, *};
use crate::engine::core::ecs::{SystemType, SystemData, SystemState, System, Entity};
use crate::engine::core::world::World;
use crate::gl;

// Create the rendering system
pub fn create_system(world: &mut World) {
	// Default render system
	let mut rs = System::default();
	rs.system_data.name = String::from("Rendering System");	
	rs.system_data.link_component::<components::Render>(world);
	rs.system_data.link_component::<transforms::Position>(world);
	rs.system_data.link_component::<transforms::Position>(world);

	// When the render system gets updated
	unsafe { 
		gl::ClearColor(0.0, 0.0, 0.0, 0.0);
		let default_size = World::get_default_window_size();
		gl::Viewport(0, 0, default_size.0, default_size.1);
		gl::Enable(gl::DEPTH_TEST);
		gl::Enable(gl::CULL_FACE);	
		gl::CullFace(gl::BACK);
	}
	// Render the entitites
	rs.system_data.entity_loop_event = |entity, world| {	
		let _id = entity.entity_id;
		let shader: &mut Shader;
		let view_matrix: glam::Mat4;
		let projection_matrix: glam::Mat4;
		// Get the projection * view matrix
		{
			let camera_entity = world.get_entity(world.default_camera_id);
			let camera_data = camera_entity.get_component::<components::Camera>(world);
			projection_matrix = camera_data.projection_matrix;
			view_matrix = camera_data.view_matrix;
		}
		let model_matrix: glam::Mat4;
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

		let loc = shader.get_uniform_location(CString::new("mvp_matrix").unwrap());
		// Calculate the mvp matrix		
		let mvp_matrix: glam::Mat4 = projection_matrix * view_matrix * model_matrix;
		// Pass the MVP to the shader
		shader.set_matrix_44_uniform(loc, mvp_matrix);

		unsafe {
			// Actually draw the array
			let rc = entity.get_component::<components::Render>(world);
			if rc.gpu_data.initialized {
				gl::BindBuffer(gl::ELEMENT_ARRAY_BUFFER, rc.gpu_data.element_buffer_object);
				gl::DrawElements(gl::TRIANGLES, rc.model.indices.len() as i32, gl::UNSIGNED_INT, null());
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
}