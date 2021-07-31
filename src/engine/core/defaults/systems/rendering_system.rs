use std::ffi::CString;
use std::ptr::null;
use crate::engine::rendering::*;
use crate::engine::core::defaults::components::{components, *};
use crate::engine::core::ecs::{SystemType, System, SystemState, SystemComponent, ComponentID, Entity};
use crate::engine::core::world::World;
use crate::gl;

// Create the rendering system component
#[derive(Default)]
pub struct RendererS {
	pub framebuffer: u32,
	pub color_texture: Texture,
	pub depth_stencil_texture: Texture,
	pub quad_renderer_index: u16,
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
	rs.link_system_component::<RendererS>(world);

	let mut quad_renderer_component = components::Renderer::default();
	quad_renderer_component.model = Model::from_resource(world.resource_manager.load_resource("screen_quad.mdl3d.pkg", "models\\").unwrap()).unwrap();;
	quad_renderer_component.shader_name = {
		// Load the shader that will draw the quad
		let shader = Shader::from_vr_fr_subshader_files("passthrough.vrsh.glsl.pkg", "screen_quad.frsh.glsl.pkg", world);
		let shader = world.shader_manager.cache_shader(shader).unwrap();
		shader.name.clone()
	};
	// Add the discrete component
	quad_renderer_component.refresh_model();
	let index = world.add_discrete_component(quad_renderer_component);
	rs.get_system_component_mut::<RendererS>(world).quad_renderer_index = index;

	// When the render system gets updated
	unsafe { 
		gl::ClearColor(0.0, 0.0, 0.0, 0.0);
		let default_size = World::get_default_window_size();
		gl::Viewport(0, 0, default_size.0, default_size.1);
		gl::Enable(gl::DEPTH_TEST);
		//gl::Enable(gl::CULL_FACE);	
		gl::CullFace(gl::BACK);
		let mut sc = rs.get_system_component_mut::<RendererS>(world);
		gl::GenFramebuffers(1, &mut sc.framebuffer);
		gl::BindFramebuffer(gl::FRAMEBUFFER, sc.framebuffer);
		// Check if the frame buffer is alright
		sc.color_texture = Texture::create_new_texture(default_size.0 as u32, default_size.1 as u32, gl::RGB, gl::RGB, gl::UNSIGNED_BYTE);
		sc.depth_stencil_texture = Texture::create_new_texture(default_size.0 as u32, default_size.1 as u32, gl::DEPTH24_STENCIL8, gl::DEPTH_STENCIL, gl::UNSIGNED_INT_24_8);
		gl::BindTexture(gl::TEXTURE_2D, sc.color_texture.id);
		gl::FramebufferTexture2D(gl::FRAMEBUFFER, gl::COLOR_ATTACHMENT0, gl::TEXTURE_2D, sc.color_texture.id, 0);
		gl::BindTexture(gl::TEXTURE_2D, sc.depth_stencil_texture.id);
		gl::FramebufferTexture2D(gl::FRAMEBUFFER, gl::DEPTH_STENCIL_ATTACHMENT, gl::TEXTURE_2D, sc.depth_stencil_texture.id, 0);
		if gl::CheckFramebufferStatus(gl::FRAMEBUFFER) == gl::FRAMEBUFFER_COMPLETE {
			println!("Framebuffer is okay :)");
		} else {
			panic!("Framebuffer has failed initialization");
		}
		gl::BindFramebuffer(gl::FRAMEBUFFER, 0);
	}
	// Before we render the scene
	rs.system_pre_loop_event = |world, _| {
		unsafe {
			gl::Clear(gl::COLOR_BUFFER_BIT | gl::DEPTH_BUFFER_BIT);
		}
	};
	// After we render the scene
	rs.system_post_loop_event = |world, system| {
		let quad_renderer = world.get_dicrete_component::<components::Renderer>(system.get_system_component::<RendererS>(world).quad_renderer_index);

		// Render the screen quad
		unsafe {
			gl::BindVertexArray(quad_renderer.gpu_data.vertex_array_object);
			gl::BindBuffer(gl::ELEMENT_ARRAY_BUFFER, quad_renderer.gpu_data.element_buffer_object);
			gl::DrawElements(gl::TRIANGLES, quad_renderer.model.triangles.len() as i32, gl::UNSIGNED_INT, null());
		}
	};
	// Render the entitites
	rs.entity_loop_event = |entity, world, _| {	
		let _id = entity.entity_id;
		let shader: &mut Shader;
		let view_matrix: glam::Mat4;
		let projection_matrix: glam::Mat4;
		let camera_position: glam::Vec3;
		// Get the projection * view matrix
		{
			let camera_entity = world.get_entity(world.default_camera_id);
			let camera_data = camera_entity.get_component::<components::Camera>(world);
			projection_matrix = camera_data.projection_matrix;
			view_matrix = camera_data.view_matrix;
			camera_position = camera_entity.get_component::<transforms::Position>(world).position;
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
		shader.set_matrix_44_uniform(shader.get_uniform_location("mvp_matrix"), mvp_matrix);
		shader.set_matrix_44_uniform(shader.get_uniform_location("model_matrix"), model_matrix);
		shader.set_matrix_44_uniform(shader.get_uniform_location("view_matrix"), view_matrix);
		shader.set_scalar_2_uniform(shader.get_uniform_location("resolution"), (world.window.size.0 as f32, world.window.size.1 as f32));
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
	rs.entity_added_event = |entity, world, _| {
		let rc = entity.get_component_mut::<components::Renderer>(world);
		// Use the default shader for this entity renderer
		// Make sure we create the OpenGL data for this entity's model
		rc.refresh_model();
	};
	// When an entity gets removed from the render system
	rs.entity_removed_event = |entity, world, _| {
		let rc = entity.get_component_mut::<components::Renderer>(world);
		rc.dispose_model();
	};
	rs.stype = SystemType::Render;
	rs.link_component::<components::Renderer>(world);
	world.add_system(rs);
}