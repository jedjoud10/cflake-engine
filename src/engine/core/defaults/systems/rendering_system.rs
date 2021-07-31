use std::ffi::CString;
use std::ptr::null;
use crate::engine::rendering::*;
use crate::engine::core::defaults::components::{components, *};
use crate::engine::core::ecs::{SystemType, System, SystemState, SystemComponent, ComponentID, Entity};
use crate::engine::core::world::World;
use crate::gl;

// Create the rendering system component
pub struct RendererS {

}

impl SystemComponent for RendererS {
    fn as_any(&self) -> &dyn std::any::Any {
        self
    }

    fn as_any_mut(&mut self) -> &mut dyn std::any::Any {
        self
    }
}

impl ComponentID for RendererS {
    fn get_component_name() -> String {
        String::from("Renderer System Component")
    }
}

// Create the rendering system
pub fn create_system(world: &mut World) {
	// Default render system
	let mut rs = System::default();
	rs.name = String::from("Rendering System");	
	rs.link_component::<components::Renderer>(world);
	rs.link_component::<transforms::Position>(world);
	rs.link_component::<transforms::Rotation>(world);
	rs.link_component::<transforms::Scale>(world);

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
	rs.entity_loop_event = |entity, world| {	
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
				let scale: f32;
				{
					position = entity.get_component::<transforms::Position>(world).position;
					rotation = entity.get_component::<transforms::Rotation>(world).rotation;
					scale = entity.get_component::<transforms::Scale>(world).scale;
				}
				let rc = entity.get_component_mut::<components::Renderer>(world);
				rc.update_model_matrix(position.clone(), rotation.clone(), scale);
				name = rc.shader_name.clone();
				model_matrix = rc.gpu_data.model_matrix.clone();
			}
			shader = world.shader_manager.get_shader(&name).unwrap();
		}
		// Use the shader, and update any uniforms
		shader.use_shader();

		// Calculate the mvp matrix		
		let mvp_matrix: glam::Mat4 = projection_matrix * view_matrix * model_matrix;
		// Pass the MVP and the model matrix to the shader
		shader.set_matrix_44_uniform(shader.get_uniform_location(CString::new("mvp_matrix").unwrap()), mvp_matrix);
		shader.set_matrix_44_uniform(shader.get_uniform_location(CString::new("model_matrix").unwrap()), model_matrix);

		unsafe {
			// Actually draw the array
			let rc = entity.get_component::<components::Renderer>(world);
			if rc.gpu_data.initialized {
				gl::BindVertexArray(rc.gpu_data.vertex_array_object);
				gl::BindBuffer(gl::ELEMENT_ARRAY_BUFFER, rc.gpu_data.element_buffer_object);
				gl::DrawElements(gl::TRIANGLES, rc.model.triangles.len() as i32, gl::UNSIGNED_INT, null());
			}
		}
	};
	// When an entity gets added to the render system
	rs.entity_added_event = |entity, world| {
		let rc = entity.get_component_mut::<components::Renderer>(world);
		// Use the default shader for this entity renderer
		// Make sure we create the OpenGL data for this entity's model
		rc.refresh_model();
	};
	// When an entity gets removed from the render system
	rs.entity_removed_event = |entity, world| {
		let rc = entity.get_component_mut::<components::Renderer>(world);
		rc.dispose_model();
	};
	rs.stype = SystemType::Render;
	rs.link_component::<components::Renderer>(world);
	world.add_system(rs);
}