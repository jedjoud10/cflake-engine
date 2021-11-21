use std::ffi::CString;
use std::ptr::null;

use assets::{AssetManager, AssetObject};

use crate::{DataType, Shader, Uniform};
use crate::{FrameStats, Material, MaterialFlags, Renderer, Texture, TextureType, pipec, pipeline::object::*};
use crate::basics::texture::*;
// The main renderer, this is stored
#[derive(Default)]
pub struct PipelineRenderer {
    pub framebuffer: u32,
    // The frame buffer textures
    pub diffuse_texture: TextureGPUObject,
    pub normals_texture: TextureGPUObject,
    pub position_texture: TextureGPUObject,
    pub depth_texture: TextureGPUObject,
    pub debug_view: u16,
    pub wireframe: bool,
    wireframe_shader: ShaderGPUObject,
    frame_stats: FrameStats,
}

// Render debug primitives
pub fn render_debug_primitives(primitives: Vec<(RendererGPUObject, veclib::Matrix4x4<f32>)>, camera: &CameraDataGPUObject) {
    let vp_m = camera.projm * camera.viewm;
    for (primitive, modelm)  in primitives.iter() {
        render(primitive, modelm, camera);
    }
}

// Render a renderer normally
pub fn render(renderer: &RendererGPUObject, model_matrix: &veclib::Matrix4x4<f32>, camera: &CameraDataGPUObject) {
    let shader = (renderer.1).0;
    let material = renderer.1;
    let model = renderer.0;
    let mut group = shader.new_uniform_group();
    // Calculate the mvp matrix
    let mvp_matrix: veclib::Matrix4x4<f32> = camera.projm * camera.viewm * *model_matrix;
    // Pass the MVP and the model matrix to the shader
    group.set_mat44("mvp_matrix", mvp_matrix);
    group.set_mat44("model_matrix", *model_matrix);
    group.set_mat44("view_matrix", camera.viewm);
    group.set_vec3f32("view_pos", camera.position);
    group.consume();

    unsafe {
        // Enable / Disable vertex culling for double sided materials
        if material.2.contains(MaterialFlags::DOUBLE_SIDED) { gl::Disable(gl::CULL_FACE); }            
        else { gl::Enable(gl::CULL_FACE); }

        // Actually draw
        gl::BindVertexArray(model.0);
        gl::BindBuffer(gl::ELEMENT_ARRAY_BUFFER, model.1);
        gl::DrawElements(gl::TRIANGLES, model.2 as i32, gl::UNSIGNED_INT, null());
    }
}

// Render a multi-material renderer
fn render_mm(renderer: &RendererGPUObject, model_matrix: &veclib::Matrix4x4<f32>, camera: &CameraDataGPUObject) {
    // TODO: Gotta reprogram the multi material system now
}

// Render a renderer using wireframe 
fn render_wireframe(renderer: &RendererGPUObject, model_matrix: &veclib::Matrix4x4<f32>, camera: &CameraDataGPUObject, ws: &ShaderGPUObject) {
    let shader = (renderer.1).0;
    let material = renderer.1;
    let model = renderer.0;
    let mut group = ws.new_uniform_group();
    // Calculate the mvp matrix
    let mvp_matrix: veclib::Matrix4x4<f32> = camera.projm * camera.viewm * *model_matrix;
    group.set_mat44("mvp_matrix", mvp_matrix);
    group.set_mat44("model_matrix", *model_matrix);
    group.set_mat44("view_matrix", camera.viewm);
    unsafe {
        // Set the wireframe rendering
        gl::PolygonMode(gl::FRONT_AND_BACK, gl::LINE);
        gl::Enable(gl::LINE_SMOOTH);

        gl::BindVertexArray(model.0);
        gl::BindBuffer(gl::ELEMENT_ARRAY_BUFFER, model.1);
        gl::DrawElements(gl::TRIANGLES, model.2 as i32, gl::UNSIGNED_INT, null());

        // Reset the wireframe settings
        gl::BindTexture(gl::TEXTURE_2D, 0);
        gl::Disable(gl::LINE_SMOOTH);
        gl::PolygonMode(gl::FRONT_AND_BACK, gl::FILL);
    }
}

impl PipelineRenderer {
    // Init the pipeline renderer
    pub fn init(&mut self, dimensions: veclib::Vector2<u16>) {
        /* #region Deferred renderer init */
        // Local function for binding a texture to a specific frame buffer attachement
        fn bind_attachement(attachement: u32, texture: &TextureGPUObject) {
            unsafe {
                // Default target, no multisamplind
                let target: u32 = gl::TEXTURE_2D;
                gl::BindTexture(target, texture.0);
                gl::FramebufferTexture2D(gl::FRAMEBUFFER, attachement, target, texture.0, 0);
            }
        }  
        unsafe {
            gl::GenFramebuffers(1, &mut self.framebuffer);
            gl::BindFramebuffer(gl::FRAMEBUFFER, self.framebuffer);
            let dims = TextureType::Texture2D(dimensions.x, dimensions.y);
            // Create the diffuse render texture
            self.diffuse_texture = pipec::texture(Texture::default()
                .set_dimensions(dims)
                .set_format(TextureFormat::RGB32F));
            // Create the normals render texture
            self.normals_texture = pipec::texture(Texture::default()
                .set_dimensions(dims)
                .set_format(TextureFormat::RGB8RS));
            // Create the position render texture
            self.position_texture = pipec::texture(Texture::default()
                .set_dimensions(dims)
                .set_format(TextureFormat::RGB32F));
            // Create the depth render texture
            self.depth_texture = pipec::texture(Texture::default()
                .set_dimensions(dims)
                .set_format(TextureFormat::DepthComponent32)
                .set_data_type(DataType::Float32));

            // Now bind the attachememnts
            bind_attachement(gl::COLOR_ATTACHMENT0, &self.diffuse_texture);
            bind_attachement(gl::COLOR_ATTACHMENT1, &self.normals_texture);
            bind_attachement(gl::COLOR_ATTACHMENT2, &self.position_texture);
            bind_attachement(gl::DEPTH_ATTACHMENT, &self.depth_texture);
            let attachements = vec![gl::COLOR_ATTACHMENT0, gl::COLOR_ATTACHMENT1, gl::COLOR_ATTACHMENT2];
            gl::DrawBuffers(attachements.len() as i32, attachements.as_ptr() as *const u32);

            // Check if the frame buffer is okay
            if gl::CheckFramebufferStatus(gl::FRAMEBUFFER) == gl::FRAMEBUFFER_COMPLETE {
            } else {
                panic!("Framebuffer has failed initialization! Error: '{}'", gl::CheckFramebufferStatus(gl::FRAMEBUFFER));
            }

            // Unbind
            gl::BindFramebuffer(gl::FRAMEBUFFER, 0);
        }
        /* #endregion */
        /* #region Actual pipeline renderer shit */
        /*
        // Load sky gradient texture
        let texture = pipec::texturec(Texture::default()
            .set_wrapping_mode(TextureWrapping::ClampToEdge)
            .cache_load("defaults\\textures\\sky_gradient.png", asset_manager));

        // Load the wireframe shader
        self.wireframe_shader = pipec::shader(Shader::default()
            .load_shader(
                vec!["defaults\\shaders\\rendering\\default.vrsh.glsl", "defaults\\shaders\\others\\wireframe.frsh.glsl"],
                asset_manager,
            ).unwrap());
        */
        /* #endregion */ 
    }
    // Pre-render event
    pub fn pre_render(&mut self) {
        unsafe {
            gl::BindFramebuffer(gl::FRAMEBUFFER, self.framebuffer);
            gl::Clear(gl::COLOR_BUFFER_BIT | gl::DEPTH_BUFFER_BIT);
        }
    }
    // Called each frame, for each renderer that is valid in the pipeline
    pub fn renderer_frame(&self, renderer: &RendererGPUObject, model_matrix: &veclib::Matrix4x4<f32>, camera: &CameraDataGPUObject) {        
        // TODO: Actually re-implement multi material renderers
        // Should we render in wireframe or not?
        if self.wireframe {
            render_wireframe(renderer, model_matrix, camera, &self.wireframe_shader);
        } else {
            render(renderer, model_matrix, camera);
        }
    }    
    // Post-render event
    pub fn post_render(&self, dimensions: veclib::Vector2<u16>, camera: &CameraDataGPUObject) {
        // Update the frame stats texture
        //self.frame_stats.update_texture(data.time_manager, &data.entity_manager.entities);
        // Render the screen QUAD
        let shader = self.quad_renderer.material.shader.as_ref().unwrap();
        shader.use_shader();
        shader.set_vec2i32("resolution", &(dimensions.into()));
        shader.set_f32("time", &(data.time_manager.seconds_since_game_start as f32));
        shader.set_vec2f32("nf_planes", &veclib::Vector2::new(camera.clip_planes.0, camera.clip_planes.1));
        shader.set_vec3f32("directional_light_dir", &data.custom_data.light_dir);
        // Textures
        shader.set_t2d("diffuse_texture", &system.diffuse_texture, 0);
        shader.set_t2d("normals_texture", &system.normals_texture, 1);
        shader.set_t2d("position_texture", &system.position_texture, 2);
        shader.set_t2d("depth_texture", &system.depth_texture, 3);
        shader.set_t2d("default_sky_gradient", data.custom_data.sky_texture.as_ref().unwrap(), 5);
        let vp_m = camera.projection_matrix * (veclib::Matrix4x4::from_quaternion(&camera_transform.rotation));
        shader.set_mat44("custom_vp_matrix", &vp_m);
        // Other params
        shader.set_vec3f32("camera_pos", &camera_transform.position);
        shader.set_i32("debug_view", &(system.debug_view as i32));
        shader.set_t2d("frame_stats", &system.frame_stats.texture, 6);
        // Render the screen quad
        unsafe {
            gl::BindFramebuffer(gl::FRAMEBUFFER, 0);
            gl::Clear(gl::COLOR_BUFFER_BIT | gl::DEPTH_BUFFER_BIT);
            gl::BindVertexArray(system.quad_renderer.gpu_data.vertex_array_object);
            gl::BindBuffer(gl::ELEMENT_ARRAY_BUFFER, system.quad_renderer.gpu_data.element_buffer_object);
            gl::DrawElements(gl::TRIANGLES, system.quad_renderer.model.triangles.len() as i32, gl::UNSIGNED_INT, null());
        }
    }
}
fn system_postfire(system_data: &mut SystemData, data: &mut WorldData) {
    
}
fn entity_added(_system_data: &mut SystemData, entity: &Entity, data: &mut WorldData) {
    let rc = entity.get_component_mut::<Renderer>(&mut data.component_manager).unwrap();
    // Make sure we create the OpenGL data for this entity's model
    rc.refresh_model();
    let transform = entity.get_component_mut::<components::Transform>(&mut data.component_manager).unwrap();
    transform.update_matrix();
}
fn entity_removed(_system_data: &mut SystemData, entity: &Entity, data: &mut WorldData) {
    let rc = entity.get_component_mut::<Renderer>(&mut data.component_manager).unwrap();
    let i = Instant::now();
    // Dispose the model when the entity gets destroyed
    rc.dispose_model();
    // Dispose of a complex model if it exists
    match rc.multi_material.as_mut() {
        Some(x) => {
            // Dispose
            x.dispose_models();
        }
        None => {}
    }
}
fn entity_update(system_data: &mut SystemData, entity: &Entity, components: &FilteredLinkedComponents, data: &mut WorldData) {
    
}