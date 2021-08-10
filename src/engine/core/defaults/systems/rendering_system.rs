use crate::engine::core::defaults::components::{components, *};
use crate::engine::core::ecs::{
    component::{Component, ComponentID, ComponentManager},
    entity::Entity,
    system::{System, SystemManager},
    system_data::{FireData, FireDataFragment, SystemData},
};
use crate::engine::core::world::{World};
use crate::engine::rendering::model::Model;
use crate::engine::rendering::renderer::Renderer;
use crate::engine::rendering::shader::Shader;
use crate::engine::rendering::texture::Texture;
use crate::engine::rendering::*;
use crate::gl;
use std::ffi::CString;
use std::ptr::null;

#[derive(Default)]
pub struct RenderingSystem {
    pub system_data: SystemData,
    pub framebuffer: u32,
    pub color_texture: Texture,
    pub normals_texture: Texture,
    pub position_texture: Texture,
    pub depth_stencil_texture: Texture,
    pub quad_renderer_index: u16,
    pub debug_view: u16,
	quad_renderer: Renderer,
    window: Window,
}

impl System for RenderingSystem {
    // Wrappers around system data
    fn get_system_data(&self) -> &SystemData {
        return &self.system_data;
    }

    fn get_system_data_mut(&mut self) -> &mut SystemData {
        return &mut self.system_data;
    }

	// When the system gets added to the world
	fn system_added(&mut self, data: &mut FireData, system_id: u8) {
		data.custom_data.render_system_id = system_id;
	}

    // Setup the system
    fn setup_system(&mut self, data: &mut FireData) {
        let mut system_data = &mut self.system_data;
        system_data.link_component::<Renderer>(&mut data.component_manager);
        system_data.link_component::<transforms::Position>(&mut data.component_manager);
        system_data.link_component::<transforms::Rotation>(&mut data.component_manager);
        system_data.link_component::<transforms::Scale>(&mut data.component_manager);

        let mut quad_renderer_component = Renderer::default();
        quad_renderer_component.model = Model::from_resource(
            data.resource_manager
                .load_resource("screen_quad.mdl3d.pkg", "models\\")
                .unwrap(),
        )
        .unwrap();
        quad_renderer_component.shader_name = {
            // Load the shader that will draw the quad
            let mut shader = Shader::from_vr_fr_subshader_files(
                "passthrough.vrsh.glsl.pkg",
                "screen_quad.frsh.glsl.pkg",
                &mut data.resource_manager, &mut data.shader_manager
            );
            shader.finalize_shader();
            let shader = data.shader_manager.cache_shader(shader).unwrap();
            shader.name.clone()
        };
        quad_renderer_component.refresh_model();
		self.quad_renderer = quad_renderer_component;

        unsafe {
            gl::ClearColor(0.0, 0.0, 0.0, 0.0);
            let default_size = World::get_default_window_size();
            gl::Viewport(0, 0, default_size.0, default_size.1);
            gl::Enable(gl::DEPTH_TEST);
            gl::Enable(gl::CULL_FACE);
            gl::CullFace(gl::BACK);
            gl::GenFramebuffers(1, &mut self.framebuffer);
            gl::BindFramebuffer(gl::FRAMEBUFFER, self.framebuffer);
            // Check if the frame buffer is alright
            // Create the color render texture
            self.color_texture = Texture::create_new_texture(
                default_size.0 as u16,
                default_size.1 as u16,
                gl::RGB,
                gl::RGB,
                gl::UNSIGNED_BYTE,
            );
            // Create the normals render texture
            self.normals_texture = Texture::create_new_texture(
                default_size.0 as u16,
                default_size.1 as u16,
                gl::RGB16_SNORM,
                gl::RGB,
                gl::UNSIGNED_BYTE,
            );
            // Create the position render texture
            self.position_texture = Texture::create_new_texture(
                default_size.0 as u16,
                default_size.1 as u16,
                gl::RGB32F,
                gl::RGB,
                gl::UNSIGNED_BYTE,
            );
            // Create the depth-stencil render texture
            self.depth_stencil_texture = Texture::create_new_texture(
                default_size.0 as u16,
                default_size.1 as u16,
                gl::DEPTH24_STENCIL8,
                gl::DEPTH_STENCIL,
                gl::UNSIGNED_INT_24_8,
            );
            // Bind the color texture to the color attachement 0 of the frame buffer
            gl::BindTexture(gl::TEXTURE_2D, self.color_texture.id);
            gl::FramebufferTexture2D(
                gl::FRAMEBUFFER,
                gl::COLOR_ATTACHMENT0,
                gl::TEXTURE_2D,
                self.color_texture.id,
                0,
            );
            // Bind the normal texture to the color attachement 1 of the frame buffer
            gl::BindTexture(gl::TEXTURE_2D, self.normals_texture.id);
            gl::FramebufferTexture2D(
                gl::FRAMEBUFFER,
                gl::COLOR_ATTACHMENT1,
                gl::TEXTURE_2D,
                self.normals_texture.id,
                0,
            );
            // Bind the position texture to the color attachement 2 of the frame buffer
            gl::BindTexture(gl::TEXTURE_2D, self.position_texture.id);
            gl::FramebufferTexture2D(
                gl::FRAMEBUFFER,
                gl::COLOR_ATTACHMENT2,
                gl::TEXTURE_2D,
                self.position_texture.id,
                0,
            );

            // Bind depth-stencil render texture
            gl::BindTexture(gl::TEXTURE_2D, self.depth_stencil_texture.id);
            gl::FramebufferTexture2D(
                gl::FRAMEBUFFER,
                gl::DEPTH_STENCIL_ATTACHMENT,
                gl::TEXTURE_2D,
                self.depth_stencil_texture.id,
                0,
            );

            // Hehe boii
            let attachements = vec![
                gl::COLOR_ATTACHMENT0,
                gl::COLOR_ATTACHMENT1,
                gl::COLOR_ATTACHMENT2,
            ];
            gl::DrawBuffers(
                attachements.len() as i32,
                attachements.as_ptr() as *const u32,
            );

            if gl::CheckFramebufferStatus(gl::FRAMEBUFFER) == gl::FRAMEBUFFER_COMPLETE {
                println!("Framebuffer is okay :)");
            } else {
                panic!("Framebuffer has failed initialization");
            }
            gl::BindFramebuffer(gl::FRAMEBUFFER, 0);
        }
    }

    // Called for each entity in the system
    fn fire_entity(&mut self, entity: &mut Entity, data: &mut FireData) {
        let _id = entity.entity_id;
        let shader: &Shader;
        let view_matrix: glam::Mat4;
        let projection_matrix: glam::Mat4;
        let camera_position: glam::Vec3;
        // Get the projection * view matrix
        {
            let camera_entity = data.entity_manager.get_entity(data.custom_data.main_camera_entity_id);
            let camera_data =
                camera_entity.get_component::<components::Camera>(&mut data.component_manager);
            projection_matrix = camera_data.projection_matrix;
            view_matrix = camera_data.view_matrix;
            camera_position = camera_entity
                .get_component::<transforms::Position>(&mut data.component_manager)
                .position;
        }
        let model_matrix: glam::Mat4;
        // Render the entity
        {
            let mut name = String::new();
            // Get the model matrix
            {
                let position: glam::Vec3;
                let rotation: glam::Quat;
                let scale: f32;
                {
                    position = entity
                        .get_component::<transforms::Position>(&mut data.component_manager)
                        .position;
                    rotation = entity
                        .get_component::<transforms::Rotation>(&mut data.component_manager)
                        .rotation;
                    scale = entity
                        .get_component::<transforms::Scale>(&mut data.component_manager)
                        .scale;
                }
                let rc = entity.get_component_mut::<Renderer>(&mut data.component_manager);
                rc.update_model_matrix(position.clone(), rotation.clone(), scale);
                name = rc.shader_name.clone();
                model_matrix = rc.gpu_data.model_matrix.clone();
            }
            shader = data.shader_manager.get_shader(&name).unwrap();
        }
        // Use the shader, and update any uniforms
        shader.use_shader();

        let rc = entity.get_component::<Renderer>(&mut data.component_manager);
        // Calculate the mvp matrix
        let mvp_matrix: glam::Mat4 = projection_matrix * view_matrix * model_matrix;
        // Pass the MVP and the model matrix to the shader
        shader.set_matrix_44_uniform(shader.get_uniform_location("mvp_matrix"), mvp_matrix);
        shader.set_matrix_44_uniform(shader.get_uniform_location("model_matrix"), model_matrix);
        shader.set_matrix_44_uniform(shader.get_uniform_location("view_matrix"), view_matrix);
        shader.set_scalar_2_uniform(
            shader.get_uniform_location("resolution"),
            (self.window.size.0 as f32, self.window.size.1 as f32),
        );
        shader.set_scalar_3_uniform(
            shader.get_uniform_location("view_pos"),
            (camera_position.x, camera_position.y, camera_position.z),
        );
        // Check if we even have a diffuse texture
        if rc.diffuse_texture_id != -1 {
            // Convert the texture id into a texture, and then into a OpenGL texture id
            let diffuse_texture = data.texture_manager.get_texture(rc.diffuse_texture_id);
            shader.set_texture2d(
                shader.get_uniform_location("diffuse_tex"),
                diffuse_texture.id,
                gl::TEXTURE0,
            );
        }
        // Check if we even have a normal texture
        if rc.normal_texture_id != -1 {
            // Convert the texture id into a texture, and then into a OpenGL texture id
            let normal_texture = data.texture_manager.get_texture(rc.normal_texture_id);
            shader.set_texture2d(
                shader.get_uniform_location("normal_tex"),
                normal_texture.id,
                gl::TEXTURE1,
            );
        }
        unsafe {
            // Actually draw the array
            if rc.gpu_data.initialized {
                gl::BindVertexArray(rc.gpu_data.vertex_array_object);
                gl::BindBuffer(gl::ELEMENT_ARRAY_BUFFER, rc.gpu_data.element_buffer_object);
                gl::DrawElements(
                    gl::TRIANGLES,
                    rc.model.triangles.len() as i32,
                    gl::UNSIGNED_INT,
                    null(),
                );
            }
        }
    }

    // Called before each fire_entity event is fired
    fn pre_fire(&mut self, data: &mut FireData) {
        unsafe {
            gl::BindFramebuffer(gl::FRAMEBUFFER, self.framebuffer);
            gl::Clear(gl::COLOR_BUFFER_BIT | gl::DEPTH_BUFFER_BIT);
        }
    }

    // Called after each fire_entity event has been fired
    fn post_fire(&mut self, data: &mut FireData) {
        let shader = data
            .shader_manager
            .get_shader(&self.quad_renderer.shader_name)
            .unwrap();
        let camera_position = data
            .entity_manager
            .get_entity(data.custom_data.main_camera_entity_id)
            .get_component::<transforms::Position>(data.component_manager)
            .position;
        shader.use_shader();
        shader.set_texture2d(
            shader.get_uniform_location("color_texture"),
            self.color_texture.id,
            gl::TEXTURE0,
        );
        shader.set_texture2d(
            shader.get_uniform_location("normals_texture"),
            self.normals_texture.id,
            gl::TEXTURE1,
        );
        shader.set_texture2d(
            shader.get_uniform_location("position_texture"),
            self.position_texture.id,
            gl::TEXTURE2,
        );
        shader.set_scalar_3_uniform(
            shader.get_uniform_location("view_pos"),
            (camera_position.x, camera_position.y, camera_position.z),
        );
        shader.set_int_uniform(
            shader.get_uniform_location("debug_view"),
            self.debug_view as i32,
        );
        // Render the screen quad
        unsafe {
            gl::BindFramebuffer(gl::FRAMEBUFFER, 0);
            gl::Clear(gl::COLOR_BUFFER_BIT | gl::DEPTH_BUFFER_BIT);
            gl::BindVertexArray(self.quad_renderer.gpu_data.vertex_array_object);
            gl::BindBuffer(
                gl::ELEMENT_ARRAY_BUFFER,
                self.quad_renderer.gpu_data.element_buffer_object,
            );
            gl::DrawElements(
                gl::TRIANGLES,
                self.quad_renderer.model.triangles.len() as i32,
                gl::UNSIGNED_INT,
                null(),
            );
        }
    }

    // When an entity gets added to this system
    fn entity_added(&mut self, entity: &Entity, data: &mut FireDataFragment) {
        let rc = entity.get_component_mut::<Renderer>(&mut data.component_manager);
        // Make sure we create the OpenGL data for this entity's model
        rc.refresh_model();
    }

    // When an entity gets removed from this system
    fn entity_removed(&mut self, entity: &Entity, data: &mut FireDataFragment) {
        let rc = entity.get_component_mut::<Renderer>(&mut data.component_manager);
        // Dispose the model when the entity gets destroyed
        rc.dispose_model();
    }

	// Turn this into "Any" so we can cast into child systems
	fn as_any(&self) -> &dyn std::any::Any {
		return self;
	}

	fn as_any_mut(&mut self) -> &mut dyn std::any::Any {
		return self;
	}
}
