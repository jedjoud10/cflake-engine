use crate::engine::core::defaults::components::*;

use crate::engine::core::ecs::component::FilteredLinkedComponents;
use crate::engine::core::ecs::{
    entity::Entity,
    system::System,
    system_data::{SystemData, SystemEventData, SystemEventDataLite},
};
use crate::engine::core::world::World;
use crate::engine::debug::{DebugRendererType, DebugRendererable, DefaultDebugRendererType};
use crate::engine::math;
use crate::engine::rendering::model::Model;
use crate::engine::rendering::optimizer::RenderOptimizer;
use crate::engine::rendering::renderer::{Renderer, RendererFlags};
use crate::engine::rendering::shader::Shader;
use crate::engine::rendering::texture::Texture;
use crate::engine::rendering::window::Window;
use crate::gl;

use std::ptr::null;

#[derive(Default)]
pub struct RenderingSystem {
    pub system_data: SystemData,
    pub framebuffer: u32,
    pub diffuse_texture: Texture,
    pub normals_texture: Texture,
    pub position_texture: Texture,
    pub emissive_texture: Texture,
    pub depth_stencil_texture: Texture,
    pub quad_renderer_index: u16,
    pub debug_view: u16,
    pub wireframe: bool,
    pub wireframe_shader_name: String,
    pub window: Window,
    pub multisampling: Option<u8>,
    quad_renderer: Renderer,
}

// Everything custom
impl RenderingSystem {
    // Create the quad that will render the render buffer
    fn create_screen_quad(&mut self, data: &mut SystemEventData) {
        let mut quad_renderer_component = Renderer::default();
        quad_renderer_component.model = Model::from_resource(data.resource_manager.load_packed_resource("models\\screen_quad.mdl3d").unwrap()).unwrap();
        quad_renderer_component.shader_name = Shader::new(
            vec!["shaders\\passthrough.vrsh.glsl", "shaders\\screen_quad.frsh.glsl"],
            &mut data.resource_manager,
            &mut data.shader_cacher,
        )
        .1;
        quad_renderer_component.refresh_model();
        self.quad_renderer = quad_renderer_component;
    }    
    // Bind a specific texture attachement to the frame buffer
    fn bind_attachement(attachement: u32, texture: &Texture) {
        unsafe {
            // Default target, no multisamplind
            let target: u32 = gl::TEXTURE_2D;
            gl::BindTexture(target, texture.id);
            gl::FramebufferTexture2D(gl::FRAMEBUFFER, attachement, target, texture.id, 0);
        }
    }
    // Setup all the settings for opengl like culling and the clear color
    fn setup_opengl(&mut self, data: &mut SystemEventData) {
        unsafe {
            gl::ClearColor(0.0, 0.0, 0.0, 0.0);
            gl::Viewport(0, 0, self.window.size.0 as i32, self.window.size.1 as i32);
            gl::Enable(gl::DEPTH_TEST);
            gl::Enable(gl::CULL_FACE);
            gl::CullFace(gl::BACK);
        }

        unsafe {
            gl::GenFramebuffers(1, &mut self.framebuffer);
            gl::BindFramebuffer(gl::FRAMEBUFFER, self.framebuffer);
            // Create the diffuse render texture
            self.diffuse_texture = Texture::new()
                .set_dimensions(self.window.size.0, self.window.size.1)
                .set_idf(gl::RGB, gl::RGB, gl::UNSIGNED_BYTE)
                .generate_texture(Vec::new());
            // Create the normals render texture
            self.normals_texture = Texture::new()
                .set_dimensions(self.window.size.0, self.window.size.1)
                .set_idf(gl::RGB16_SNORM, gl::RGB, gl::UNSIGNED_BYTE)
                .generate_texture(Vec::new());
            // Create the position render texture
            self.position_texture = Texture::new()
                .set_dimensions(self.window.size.0, self.window.size.1)
                .set_idf(gl::RGB32F, gl::RGB, gl::UNSIGNED_BYTE)
                .generate_texture(Vec::new());
            // Create the emissive render texture
            self.emissive_texture = Texture::new()
                .set_dimensions(self.window.size.0, self.window.size.1)
                .set_idf(gl::RGB32F, gl::RGB, gl::UNSIGNED_BYTE)
                .generate_texture(Vec::new());
            // Create the depth-stencil render texture
            self.depth_stencil_texture = Texture::new()
                .set_dimensions(self.window.size.0, self.window.size.1)
                .set_idf(gl::DEPTH24_STENCIL8, gl::DEPTH_STENCIL, gl::UNSIGNED_INT_24_8)
                .generate_texture(Vec::new());
            // Bind the color texture to the color attachement 0 of the frame buffer
            Self::bind_attachement(gl::COLOR_ATTACHMENT0, &self.diffuse_texture);
            // Bind the normal texture to the color attachement 1 of the frame buffer
            Self::bind_attachement(gl::COLOR_ATTACHMENT1, &self.normals_texture);
            // Bind the position texture to the color attachement 2 of the frame buffer
            Self::bind_attachement(gl::COLOR_ATTACHMENT2, &self.position_texture);
            // Bind the emissive texture to the color attachement 3 of the frame buffer
            Self::bind_attachement(gl::COLOR_ATTACHMENT3, &self.emissive_texture);
            // Bind the depth/stenicl texture to the color attachement depth-stencil of the frame buffer
            Self::bind_attachement(gl::DEPTH_STENCIL_ATTACHMENT, &self.depth_stencil_texture);

            let attachements = vec![
                gl::COLOR_ATTACHMENT0,
                gl::COLOR_ATTACHMENT1,
                gl::COLOR_ATTACHMENT2,
                gl::COLOR_ATTACHMENT3,
                gl::COLOR_ATTACHMENT4,
            ];
            // Set the frame buffer attachements
            gl::DrawBuffers(attachements.len() as i32, attachements.as_ptr() as *const u32);

            // Check if the frame buffer is okay
            if gl::CheckFramebufferStatus(gl::FRAMEBUFFER) == gl::FRAMEBUFFER_COMPLETE {
            } else {
                panic!("Framebuffer has failed initialization! Error: '{}'", gl::CheckFramebufferStatus(gl::FRAMEBUFFER));
            }

            // Unbind
            gl::BindFramebuffer(gl::FRAMEBUFFER, 0);
        }

        // Setup the debug renderer
        data.debug.setup_debug_renderer(data.resource_manager, data.shader_cacher);
    }
}

impl System for RenderingSystem {
    // Wrappers around system data
    fn get_system_data(&self) -> &SystemData {
        &self.system_data
    }

    fn get_system_data_mut(&mut self) -> &mut SystemData {
        &mut self.system_data
    }

    // When the system gets added to the world
    fn system_added(&mut self, data: &mut SystemEventData, system_id: u8) {
        data.custom_data.render_system_id = system_id;
    }

    // Setup the system
    fn setup_system(&mut self, data: &mut SystemEventData) {
        self.multisampling = None;
        let system_data = self.get_system_data_mut();
        system_data.link_component::<Renderer>(data.component_manager).unwrap();
        system_data.link_component::<transforms::Position>(data.component_manager).unwrap();
        system_data.link_component::<transforms::Rotation>(data.component_manager).unwrap();
        system_data.link_component::<transforms::Scale>(data.component_manager).unwrap();
        system_data.link_component::<components::AABB>(data.component_manager).unwrap();

        // Create the screen quad
        self.create_screen_quad(data);

        // Then setup opengl and the render buffer
        let _default_size = World::get_default_window_size();
        self.setup_opengl(data);
        self.add_eppf(Box::new(RenderOptimizer::default()));

        // Load the wireframe shader
        let wireframe_shader_name = Shader::new(
            vec!["shaders\\default.vrsh.glsl", "shaders\\wireframe.frsh.glsl"],
            &mut data.resource_manager,
            &mut data.shader_cacher,
        )
        .1;
        self.wireframe_shader_name = wireframe_shader_name;
    }

    // Called for each entity in the system
    fn fire_entity(&mut self, components: &FilteredLinkedComponents, data: &mut SystemEventData) {
        let shader: &Shader;
        let view_matrix: glam::Mat4;
        let projection_matrix: glam::Mat4;
        let camera_position: glam::Vec3;
        let camera_data: &components::Camera;
        // Get everything related to the camera
        {
            let camera_entity = data.entity_manager.get_entity(&data.custom_data.main_camera_entity_id).unwrap();
            camera_data = camera_entity.get_component::<components::Camera>(&mut data.component_manager).unwrap();
            projection_matrix = camera_data.projection_matrix;
            view_matrix = camera_data.view_matrix;
            camera_position = camera_entity.get_component::<transforms::Position>(&mut data.component_manager).unwrap().position;
        }
        let model_matrix: glam::Mat4;
        // Render the entity
        {
            let name: String;
            // Get the model matrix
            {
                let position: glam::Vec3;
                let rotation: glam::Quat;
                let scale: f32;
                {
                    position = components.get_component::<transforms::Position>(data.component_manager).unwrap().position;
                    rotation = components.get_component::<transforms::Rotation>(data.component_manager).unwrap().rotation;
                    scale = components.get_component::<transforms::Scale>(data.component_manager).unwrap().scale;
                }
                let rc = components.get_component_mut::<Renderer>(&mut data.component_manager).unwrap();
                rc.update_model_matrix(position, rotation, scale);
                name = rc.shader_name.clone();
                model_matrix = rc.gpu_data.model_matrix;
            }
            shader = data.shader_cacher.1.get_object(&name).unwrap();
        }
        // Use the shader, and update any uniforms
        shader.use_shader();

        let rc = components.get_component::<Renderer>(&mut data.component_manager).unwrap();
        // Calculate the mvp matrix
        let mvp_matrix: glam::Mat4 = projection_matrix * view_matrix * model_matrix;        
        // Pass the MVP and the model matrix to the shader
        shader.set_matrix_44_uniform("mvp_matrix", mvp_matrix);
        shader.set_matrix_44_uniform("model_matrix", model_matrix);
        shader.set_matrix_44_uniform("view_matrix", view_matrix);
        shader.set_scalar_3_uniform("view_pos", (camera_position.x, camera_position.y, camera_position.z));
        shader.set_scalar_2_uniform("uv_scale", (rc.uv_scale.x, rc.uv_scale.y));
        shader.set_scalar_1_uniform("time", data.time_manager.seconds_since_game_start as f32);

        // Get the OpenGL texture id so we can bind it to the shader
        let mut textures: Vec<&Texture> = Vec::new();

        // Load the default ones
        for &id in rc.texture_cache_ids.iter() {
            // If this is a negative number, it means we've gotta use the default texture
            textures.push(data.texture_cacher.id_get_object(id).unwrap());
        }
        shader.set_texture2d("diffuse_tex", textures[0], gl::TEXTURE0);
        shader.set_texture2d("normals_tex", textures[1], gl::TEXTURE1);

        // Draw normally
        if !self.wireframe && rc.gpu_data.initialized {
            unsafe {
                // Actually draw the array
                gl::BindVertexArray(rc.gpu_data.vertex_array_object);
                gl::BindBuffer(gl::ELEMENT_ARRAY_BUFFER, rc.gpu_data.element_buffer_object);

                gl::DrawElements(gl::TRIANGLES, rc.model.triangles.len() as i32, gl::UNSIGNED_INT, null());
            }
        }
        // Draw the wireframe
        if self.wireframe && rc.gpu_data.initialized && rc.flags.contains(RendererFlags::WIREFRAME) {
            let wireframe_shader = data.shader_cacher.1.get_object(&self.wireframe_shader_name).unwrap();
            wireframe_shader.use_shader();
            wireframe_shader.set_matrix_44_uniform("mvp_matrix", mvp_matrix);
            wireframe_shader.set_matrix_44_uniform("model_matrix", model_matrix);
            wireframe_shader.set_matrix_44_uniform("view_matrix", view_matrix);
            unsafe {
                gl::PolygonMode(gl::FRONT, gl::LINE);
                gl::BindVertexArray(rc.gpu_data.vertex_array_object);
                gl::BindBuffer(gl::ELEMENT_ARRAY_BUFFER, rc.gpu_data.element_buffer_object);
                gl::DrawElements(gl::TRIANGLES, rc.model.triangles.len() as i32, gl::UNSIGNED_INT, null());
                gl::BindTexture(gl::TEXTURE_2D, 0);
                gl::PolygonMode(gl::FRONT_AND_BACK, gl::FILL);
            }
        }        
    }

    // Called before each fire_entity event is fired
    fn pre_fire(&mut self, _data: &mut SystemEventData) {
        unsafe {
            gl::BindFramebuffer(gl::FRAMEBUFFER, self.framebuffer);
            gl::Clear(gl::COLOR_BUFFER_BIT | gl::DEPTH_BUFFER_BIT);
        }
    }

    // Called after each fire_entity event has been fired
    fn post_fire(&mut self, data: &mut SystemEventData) {
        // At the end of each frame, disable the depth test and render the debug objects
        let mut vp_matrix: glam::Mat4;
        let frustum: &math::Frustum;
        // Get the (projection * view) matrix
        {
            let camera_entity = data.entity_manager.get_entity(&data.custom_data.main_camera_entity_id).unwrap();
            let camera_data = camera_entity.get_component::<components::Camera>(&mut data.component_manager).unwrap();
            let projection_matrix = camera_data.projection_matrix;
            let view_matrix = camera_data.view_matrix;
            frustum = &camera_data.frustum;
            vp_matrix = projection_matrix * view_matrix;
        }
        // Casually just draw the frustum
        data.debug.debug(frustum.get_debug_renderer());
        // Draw the debug primitives
        data.debug.draw_debug(vp_matrix, &data.shader_cacher.1);
        let shader = data.shader_cacher.1.get_object(&self.quad_renderer.shader_name).unwrap();
        let camera_position = data
            .entity_manager
            .get_entity(&data.custom_data.main_camera_entity_id)
            .unwrap()
            .get_component::<transforms::Position>(data.component_manager)
            .unwrap()
            .position;
        shader.use_shader();
        shader.set_texture2d("diffuse_texture", &self.diffuse_texture, gl::TEXTURE0);
        shader.set_texture2d("normals_texture", &self.normals_texture, gl::TEXTURE1);
        shader.set_texture2d("position_texture", &self.position_texture, gl::TEXTURE2);
        shader.set_texture2d("emissive_texture", &self.emissive_texture, gl::TEXTURE3);
        shader.set_scalar_2_uniform("resolution", (self.window.size.0 as f32, self.window.size.1 as f32));
        shader.set_scalar_1_uniform("time", data.time_manager.seconds_since_game_start as f32);
        // Sky params
        shader.set_scalar_3_uniform("directional_light_dir", (0.0, 1.0, 0.0));
        let sky_component = data
            .entity_manager
            .get_entity(&data.custom_data.sky_entity_id)
            .unwrap()
            .get_component::<components::Sky>(data.component_manager)
            .unwrap();

        // Set the sky gradient
        shader.set_texture2d(
            "default_sky_gradient",
            data.texture_cacher.id_get_object(sky_component.sky_gradient_texture_id).unwrap(),
            gl::TEXTURE4,
        );

        // Other params
        shader.set_scalar_3_uniform("view_pos", (camera_position.x, camera_position.y, camera_position.z));
        shader.set_int_uniform("debug_view", self.debug_view as i32);
        shader.set_scalar_2_uniform("resolution", (self.window.size.0 as f32, self.window.size.1 as f32));
        // Render the screen quad
        unsafe {
            gl::BindFramebuffer(gl::FRAMEBUFFER, 0);
            gl::Clear(gl::COLOR_BUFFER_BIT | gl::DEPTH_BUFFER_BIT);
            gl::BindVertexArray(self.quad_renderer.gpu_data.vertex_array_object);
            gl::BindBuffer(gl::ELEMENT_ARRAY_BUFFER, self.quad_renderer.gpu_data.element_buffer_object);
            gl::DrawElements(gl::TRIANGLES, self.quad_renderer.model.triangles.len() as i32, gl::UNSIGNED_INT, null());
        }
    }

    // When an entity gets added to this system
    fn entity_added(&mut self, entity: &Entity, data: &mut SystemEventDataLite) {
        let rc = entity.get_component_mut::<Renderer>(&mut data.component_manager).unwrap();
        // Make sure we create the OpenGL data for this entity's model
        rc.refresh_model();
    }

    // When an entity gets removed from this system
    fn entity_removed(&mut self, entity: &Entity, data: &mut SystemEventDataLite) {
        let rc = entity.get_component_mut::<Renderer>(&mut data.component_manager).unwrap();
        // Dispose the model when the entity gets destroyed
        rc.dispose_model();
    }

    // Turn this into "Any" so we can cast into child systems
    fn as_any(&self) -> &dyn std::any::Any {
        self
    }

    fn as_any_mut(&mut self) -> &mut dyn std::any::Any {
        self
    }
}
