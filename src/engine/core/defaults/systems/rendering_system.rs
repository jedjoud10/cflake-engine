use crate::engine::core::defaults::components::{components, *};
use crate::engine::core::ecs::component;
use crate::engine::core::ecs::{
    component::{Component, ComponentID, ComponentManager},
    entity::Entity,
    system::{System, SystemManager},
    system_data::{SystemData, SystemEventData, SystemEventDataLite},
};
use crate::engine::core::world::World;
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
	pub framebuffer_ms: u32,
    pub diffuse_texture: Texture,
    pub normals_texture: Texture,
    pub position_texture: Texture,
    pub emissive_texture: Texture,
    pub depth_stencil_texture: Texture,
    pub quad_renderer_index: u16,
    pub debug_view: u16,
    pub wireframe: bool,
    pub wireframe_shader_name: String,
    quad_renderer: Renderer,
    window: Window,
}

impl RenderingSystem {
    // Create the quad that will render the render buffer
    fn create_screen_quad(&mut self, data: &mut SystemEventData) {
        let mut quad_renderer_component = Renderer::default();
        quad_renderer_component.model = Model::from_resource(
            data.resource_manager
                .load_packed_resource("models\\screen_quad.mdl3d")
                .unwrap(),
        )
        .unwrap();
        quad_renderer_component.shader_name = Shader::new(
            vec![
                "shaders\\passthrough.vrsh.glsl",
                "shaders\\screen_quad.frsh.glsl",
            ],
            &mut data.resource_manager,
            &mut data.shader_cacher,
        )
        .1;
        quad_renderer_component.refresh_model();
        self.quad_renderer = quad_renderer_component;
    }

    // Setup all the settings for opengl like culling and the clear color
    fn setup_opengl(&mut self, default_size: (i32, i32)) {
        unsafe {
            gl::ClearColor(0.0, 0.0, 0.0, 0.0);
            gl::Viewport(0, 0, default_size.0, default_size.1);
            gl::Enable(gl::DEPTH_TEST);
            gl::Enable(gl::CULL_FACE);
			gl::Enable(gl::MULTISAMPLE);
            gl::CullFace(gl::BACK);
        }
    }

	// Bind a specific texture attachement to the frame buffer
	fn bind_attachement(attachement: u32, multisampled: bool, texture: u32) {
		unsafe {
			// Default target, no multisamplind
			let mut target: u32 = gl::TEXTURE_2D;
			if multisampled {
				// Activate multisampling for this attachement
				target = gl::TEXTURE_2D_MULTISAMPLE;
			}
			gl::BindTexture(target, texture);
            gl::FramebufferTexture2D(
                gl::FRAMEBUFFER,
                attachement,
                target,
                texture,
                0,
            );
		}
	}

    // Setup the render buffer
    fn setup_render_buffer(&mut self, default_size: (i32, i32)) {
        unsafe {
			// This framebuffer will hold the multisampled textures
            let default_size: (u16, u16) = (default_size.0 as u16, default_size.1 as u16);
            gl::GenFramebuffers(1, &mut self.framebuffer_ms);
            gl::BindFramebuffer(gl::FRAMEBUFFER, self.framebuffer_ms);
            // Create the diffuse render texture
            let diffuse_texture = Texture::new()
                .set_dimensions(default_size.0, default_size.1)
                .set_idf(gl::RGB, gl::RGB, gl::UNSIGNED_BYTE)
				.enable_multisampling(4)
                .generate_texture(Vec::new());
            // Create the normals render texture
            let normals_texture = Texture::new()
                .set_dimensions(default_size.0, default_size.1)
                .set_idf(gl::RGB16_SNORM, gl::RGB, gl::UNSIGNED_BYTE)
				.enable_multisampling(4)
                .generate_texture(Vec::new());
            // Create the position render texture
            let position_texture = Texture::new()
                .set_dimensions(default_size.0, default_size.1)
                .set_idf(gl::RGB32F, gl::RGB, gl::UNSIGNED_BYTE)
				.enable_multisampling(4)
                .generate_texture(Vec::new());
            // Create the emissive render texture
            let emissive_texture = Texture::new()
                .set_dimensions(default_size.0, default_size.1)
                .set_idf(gl::RGB32F, gl::RGB, gl::UNSIGNED_BYTE)
				.enable_multisampling(4)
                .generate_texture(Vec::new());
            // Create the depth-stencil render texture
            let depth_stencil_texture = Texture::new()
                .set_dimensions(default_size.0, default_size.1)
                .set_idf(
                    gl::DEPTH24_STENCIL8,
                    gl::DEPTH_STENCIL,
                    gl::UNSIGNED_INT_24_8,
                )
				.enable_multisampling(4)
                .generate_texture(Vec::new());
            // Bind the color texture to the color attachement 0 of the frame buffer
            Self::bind_attachement(gl::COLOR_ATTACHMENT0, true, diffuse_texture.id);
            // Bind the normal texture to the color attachement 1 of the frame buffer
			Self::bind_attachement(gl::COLOR_ATTACHMENT1, true, normals_texture.id);
            // Bind the position texture to the color attachement 2 of the frame buffer
			Self::bind_attachement(gl::COLOR_ATTACHMENT2, true, position_texture.id);
            // Bind the emissive texture to the color attachement 3 of the frame buffer
			Self::bind_attachement(gl::COLOR_ATTACHMENT3, true, emissive_texture.id);
            
            // Create a depth-stencil render object and attach it to the framebuffer
			let mut depth_stencil_renderobject = 0;
			gl::GenRenderbuffers(1, &mut depth_stencil_renderobject);
			gl::BindRenderbuffer(gl::RENDERBUFFER, depth_stencil_renderobject);
			gl::RenderbufferStorageMultisample(gl::RENDERBUFFER, 4, gl::DEPTH24_STENCIL8, default_size.0 as i32, default_size.1 as i32);
			gl::BindRenderbuffer(gl::RENDERBUFFER, 0);
			gl::FramebufferRenderbuffer(gl::FRAMEBUFFER, gl::DEPTH_STENCIL_ATTACHMENT, gl::RENDERBUFFER, depth_stencil_renderobject);

            let attachements = vec![
                gl::COLOR_ATTACHMENT0,
                gl::COLOR_ATTACHMENT1,
                gl::COLOR_ATTACHMENT2,
                gl::COLOR_ATTACHMENT3,
            ];
            // Set the frame buffer attachements
            gl::DrawBuffers(
                attachements.len() as i32,
                attachements.as_ptr() as *const u32,
            );

            // Check if the frame buffer is okay
            println!("Checking multisampled framebuffer...");
            if gl::CheckFramebufferStatus(gl::FRAMEBUFFER) == gl::FRAMEBUFFER_COMPLETE {
                println!("Multisampled framebuffer is okay :)");
            } else {
                panic!(
                    "Multisampled framebuffer has failed initialization! Error: '{}'",
                    gl::CheckFramebufferStatus(gl::FRAMEBUFFER)
                );
            }
			// Unbind
            gl::BindFramebuffer(gl::FRAMEBUFFER, 0);
        }
		unsafe {
			// This framebuffer is the one that will be outputed to the screen
            let default_size: (u16, u16) = (default_size.0 as u16, default_size.1 as u16);
            gl::GenFramebuffers(1, &mut self.framebuffer);
            gl::BindFramebuffer(gl::FRAMEBUFFER, self.framebuffer);
            // Create the diffuse render texture
            self.diffuse_texture = Texture::new()
                .set_dimensions(default_size.0, default_size.1)
                .set_idf(gl::RGB, gl::RGB, gl::UNSIGNED_BYTE)
                .generate_texture(Vec::new());
            // Create the normals render texture
            self.normals_texture = Texture::new()
                .set_dimensions(default_size.0, default_size.1)
                .set_idf(gl::RGB16_SNORM, gl::RGB, gl::UNSIGNED_BYTE)
                .generate_texture(Vec::new());
            // Create the position render texture
            self.position_texture = Texture::new()
                .set_dimensions(default_size.0, default_size.1)
                .set_idf(gl::RGB32F, gl::RGB, gl::UNSIGNED_BYTE)
                .generate_texture(Vec::new());
            // Create the emissive render texture
            self.emissive_texture = Texture::new()
                .set_dimensions(default_size.0, default_size.1)
                .set_idf(gl::RGB32F, gl::RGB, gl::UNSIGNED_BYTE)
				.generate_texture(Vec::new());
            // Create the depth-stencil render texture
            self.depth_stencil_texture = Texture::new()
                .set_dimensions(default_size.0, default_size.1)
                .set_idf(
                    gl::DEPTH24_STENCIL8,
                    gl::DEPTH_STENCIL,
                    gl::UNSIGNED_INT_24_8,
                )
                .generate_texture(Vec::new());
            // Bind the color texture to the color attachement 0 of the frame buffer
            Self::bind_attachement(gl::COLOR_ATTACHMENT0, false, self.diffuse_texture.id);
            // Bind the normal texture to the color attachement 1 of the frame buffer
			Self::bind_attachement(gl::COLOR_ATTACHMENT1, false, self.normals_texture.id);
            // Bind the position texture to the color attachement 2 of the frame buffer
			Self::bind_attachement(gl::COLOR_ATTACHMENT2, false, self.position_texture.id);
            // Bind the emissive texture to the color attachement 3 of the frame buffer
			Self::bind_attachement(gl::COLOR_ATTACHMENT3, false, self.emissive_texture.id);

			
            let attachements = vec![
                gl::COLOR_ATTACHMENT0,
                gl::COLOR_ATTACHMENT1,
                gl::COLOR_ATTACHMENT2,
                gl::COLOR_ATTACHMENT3,
            ];
            // Set the frame buffer attachements
            gl::DrawBuffers(
                attachements.len() as i32,
                attachements.as_ptr() as *const u32,
            );

            // Check if the frame buffer is okay
            println!("Checking framebuffer...");
            if gl::CheckFramebufferStatus(gl::FRAMEBUFFER) == gl::FRAMEBUFFER_COMPLETE {
                println!("Framebuffer is okay :)");
            } else {
                panic!(
                    "Framebuffer has failed initialization! Error: '{}'",
                    gl::CheckFramebufferStatus(gl::FRAMEBUFFER)
                );
            }
            gl::BindFramebuffer(gl::FRAMEBUFFER, 0);
        }    
	}
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
    fn system_added(&mut self, data: &mut SystemEventData, system_id: u8) {
        data.custom_data.render_system_id = system_id;
    }

    // Setup the system
    fn setup_system(&mut self, data: &mut SystemEventData) {
        let system_data = &mut self.system_data;
        system_data.link_component::<Renderer>(&mut data.component_manager);
        system_data.link_component::<transforms::Position>(&mut data.component_manager);
        system_data.link_component::<transforms::Rotation>(&mut data.component_manager);
        system_data.link_component::<transforms::Scale>(&mut data.component_manager);

        // Create the screen quad
        self.create_screen_quad(data);

        // Then setup opengl and the render buffer
        let default_size = World::get_default_window_size();
        self.setup_opengl(default_size);
        self.setup_render_buffer(default_size);

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
    fn fire_entity(&mut self, entity: &mut Entity, data: &mut SystemEventData) {
        let _id = entity.entity_id;
        let shader: &Shader;
        let view_matrix: glam::Mat4;
        let projection_matrix: glam::Mat4;
        let camera_position: glam::Vec3;
        // Get the projection * view matrix
        {
            let camera_entity = data
                .entity_manager
                .get_entity(data.custom_data.main_camera_entity_id)
                .unwrap();
            let camera_data = camera_entity
                .get_component::<components::Camera>(&mut data.component_manager)
                .unwrap();
            projection_matrix = camera_data.projection_matrix;
            view_matrix = camera_data.view_matrix;
            camera_position = camera_entity
                .get_component::<transforms::Position>(&mut data.component_manager)
                .unwrap()
                .position;
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
                    position = entity
                        .get_component::<transforms::Position>(&mut data.component_manager)
                        .unwrap()
                        .position;
                    rotation = entity
                        .get_component::<transforms::Rotation>(&mut data.component_manager)
                        .unwrap()
                        .rotation;
                    scale = entity
                        .get_component::<transforms::Scale>(&mut data.component_manager)
                        .unwrap()
                        .scale;
                }
                let rc = entity
                    .get_component_mut::<Renderer>(&mut data.component_manager)
                    .unwrap();
                rc.update_model_matrix(position.clone(), rotation.clone(), scale);
                name = rc.shader_name.clone();
                model_matrix = rc.gpu_data.model_matrix.clone();
            }
            shader = data.shader_cacher.1.get_object(&name).unwrap();
        }
        // Use the shader, and update any uniforms
        shader.use_shader();

        let rc = entity
            .get_component::<Renderer>(&mut data.component_manager)
            .unwrap();
        // Calculate the mvp matrix
        let mvp_matrix: glam::Mat4 = projection_matrix * view_matrix * model_matrix;
        // Pass the MVP and the model matrix to the shader
        shader.set_matrix_44_uniform("mvp_matrix", mvp_matrix);
        shader.set_matrix_44_uniform("model_matrix", model_matrix);
        shader.set_matrix_44_uniform("view_matrix", view_matrix);        
        shader.set_scalar_3_uniform(
            "view_pos",
            (camera_position.x, camera_position.y, camera_position.z),
        );
        shader.set_scalar_2_uniform("uv_scale", (rc.uv_scale.x, rc.uv_scale.y));

        // Get the OpenGL texture id so we can bind it to the shader
        let mut opengl_texture_id: Vec<u32> = Vec::new();

        // Load the default ones
        for (i, &id) in rc.texture_cache_ids.iter().enumerate() {
            // If this is a negative number, it means we've gotta use the default texture
            opengl_texture_id.push(data.texture_cacher.id_get_object(id).unwrap().id as u32);
        }
        shader.set_texture2d("diffuse_tex", opengl_texture_id[0], gl::TEXTURE0);
        shader.set_texture2d("normals_tex", opengl_texture_id[1], gl::TEXTURE1);

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
				
                // Make another other pass to render the wireframe on top
                if true {
                    let wireframe_shader = data
                        .shader_cacher
                        .1
                        .get_object(&self.wireframe_shader_name)
                        .unwrap();
                    wireframe_shader.use_shader();
                    wireframe_shader.set_matrix_44_uniform("mvp_matrix", mvp_matrix);
                    wireframe_shader.set_matrix_44_uniform("model_matrix", model_matrix);
                    wireframe_shader.set_matrix_44_uniform("view_matrix", view_matrix);
                    gl::Enable(gl::POLYGON_OFFSET_LINE);
                    gl::PolygonMode(gl::FRONT, gl::LINE);
                    gl::PolygonOffset(0.0, -1.0);
                    gl::DrawElements(
                        gl::TRIANGLES,
                        rc.model.triangles.len() as i32,
                        gl::UNSIGNED_INT,
                        null(),
                    );
					gl::BindTexture(gl::TEXTURE_2D, 0);
                    gl::PolygonMode(gl::FRONT_AND_BACK, gl::FILL);
                    gl::Disable(gl::POLYGON_OFFSET_LINE);
                }
            }
        }
    }

    // Called before each fire_entity event is fired
    fn pre_fire(&mut self, data: &mut SystemEventData) {
        unsafe {
            gl::BindFramebuffer(gl::FRAMEBUFFER, self.framebuffer_ms);
            gl::Clear(gl::COLOR_BUFFER_BIT | gl::DEPTH_BUFFER_BIT);
        }
    }

    // Called after each fire_entity event has been fired
    fn post_fire(&mut self, data: &mut SystemEventData) {
        let shader = data
            .shader_cacher
            .1
            .get_object(&self.quad_renderer.shader_name)
            .unwrap();
        let camera_position = data
            .entity_manager
            .get_entity(data.custom_data.main_camera_entity_id)
            .unwrap()
            .get_component::<transforms::Position>(data.component_manager)
            .unwrap()
            .position;
			
		unsafe {
			// Read the multisampled framebuffer and draw it to the normal framebuffer (blitting)
			gl::BindFramebuffer(gl::READ_FRAMEBUFFER, self.framebuffer_ms);
			gl::BindFramebuffer(gl::DRAW_FRAMEBUFFER, self.framebuffer);
			// Helps us a lil
			let resolution: glam::IVec2 = glam::ivec2(self.window.size.0 as i32, self.window.size.1 as i32);
			gl::BlitFramebuffer(0, 0, resolution.x, resolution.y, 0, 0, resolution.x, resolution.y, gl::COLOR_BUFFER_BIT, gl::NEAREST);
		}        
		shader.use_shader();
			let light_dir = data
            .custom_data
            .sun_rotation
            .mul_vec3(glam::vec3(0.0, 1.0, 0.0));

        // Sky params
        shader.set_scalar_3_uniform("directional_light_dir", (0.0, 1.0, 0.0));
        //shader.set_scalar_3_uniform("directional_light_dir", (light_dir.x, light_dir.y, light_dir.z));
        let sky_component = data
            .component_manager
            .id_get_component::<components::Sky>(data.custom_data.sky_component_id)
            .unwrap();
        shader.set_texture2d(
            "default_sky_gradient",
            data.texture_cacher
                .id_get_object(sky_component.sky_gradient_texture_id)
                .unwrap()
                .id,
            gl::TEXTURE5,
        );

        // Other params
        shader.set_scalar_3_uniform(
            "view_pos",
            (camera_position.x, camera_position.y, camera_position.z),
        );
        shader.set_int_uniform("debug_view", self.debug_view as i32);
		shader.set_scalar_2_uniform(
            "resolution",
            (self.window.size.0 as f32, self.window.size.1 as f32),
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
    fn entity_added(&mut self, entity: &Entity, data: &mut SystemEventDataLite) {
        let rc = entity
            .get_component_mut::<Renderer>(&mut data.component_manager)
            .unwrap();
        // Make sure we create the OpenGL data for this entity's model
        rc.refresh_model();
    }

    // When an entity gets removed from this system
    fn entity_removed(&mut self, entity: &Entity, data: &mut SystemEventDataLite) {
        let rc = entity
            .get_component_mut::<Renderer>(&mut data.component_manager)
            .unwrap();
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
