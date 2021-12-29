use crate::{pipec, pipeline::object::*, MaterialFlags, Shader, Texture};
use crate::{texture::*, DataType, GPUObjectID, Material, Window};

use glfw::Context;
use others::SmartList;
use std::collections::HashSet;
use std::ptr::null;

use super::buffer::PipelineBuffer;

// These should be ran on the main thread btw
pub mod window_commands {
    // Set fullscreen
    pub fn set_fullscreen(fullscreen: bool, glfw: &mut glfw::Glfw, window: &mut glfw::Window) {
        if fullscreen {
            // Set the glfw window as a fullscreen window
            glfw.with_primary_monitor_mut(|_glfw2, monitor| {
                let videomode = monitor.unwrap().get_video_mode().unwrap();
                window.set_monitor(glfw::WindowMode::FullScreen(monitor.unwrap()), 0, 0, videomode.width, videomode.height, None);
                unsafe {
                    // Update the OpenGL viewport
                    gl::Viewport(0, 0, videomode.width as i32, videomode.height as i32);
                }
            });
        } else {
            // Set the glfw window as a windowed window
            glfw.with_primary_monitor_mut(|_glfw2, monitor| {
                let _videomode = monitor.unwrap().get_video_mode().unwrap();
                let size = crate::WINDOW_SIZE;
                window.set_monitor(glfw::WindowMode::Windowed, 50, 50, size.x as u32, size.y as u32, None);
                unsafe {
                    // Update the OpenGL viewport
                    gl::Viewport(0, 0, size.x as i32, size.y as i32);
                }
            });
        }
        crate::pipec::task(crate::pipec::RenderTask::WindowUpdateFullscreen(fullscreen));
    }
    // Set vsync
    pub fn set_vsync(vsync: bool) {
        crate::pipec::task(crate::pipec::RenderTask::WindowUpdateVSync(vsync));
    }
    // Hide the cursor
    pub fn hide_cursor(window: &mut glfw::Window) {
        window.set_cursor_mode(glfw::CursorMode::Disabled);
        window.set_cursor_pos(0.0, 0.0);
    }
}
// The main renderer, this is stored
#[derive(Default)]
pub struct PipelineRenderer {
    framebuffer: u32,              // The master frame buffer
    diffuse_texture: GPUObjectID,  // Diffuse texture, can also store HDR values
    normals_texture: GPUObjectID,  // World Normals texture
    position_texture: GPUObjectID, // World Positions texture
    depth_texture: GPUObjectID,    // Depth texture
    debug_view: u16,               // OUr currenty debug view mode
    wireframe: bool,               // Are we rendering in wireframe or not
    quad_model: GPUObjectID,       // The current screen quad model that we are using
    screen_shader: GPUObjectID,    // The current screen quad shader that we are using
    sky_texture: GPUObjectID,      // The sky gradient texture
    wireframe_shader: GPUObjectID, // The current wireframe shader
    //frame_stats: FrameStats,                   // Some frame stats
    pub window: Window,                        // Window
    pub default_material: Option<GPUObjectID>, // Self explanatory
}

// Render debug primitives
pub fn render_debug_primitives(primitives: Vec<RendererGPUObject>, camera: &CameraDataGPUObject, dm: &MaterialGPUObject) {
    /*
    let _vp_m = camera.projm * camera.viewm;
    for primitive in &primitives {
        render(primitive, camera, dm);
    }
    */
}

// Render a renderer normally
pub fn render(buf: &PipelineBuffer, renderer: &RendererGPUObject, camera: &CameraDataGPUObject, dm: &MaterialGPUObject, new_time: f32, delta: f32, resolution: veclib::Vector2<i32>) {
    let material = buf.as_material(&renderer.material_id);
    let mut shader = buf.as_shader(&dm.shader.as_ref().unwrap()).unwrap();
    // If we do not have a material assigned, use the default material
    let material = match material {
        Some(user_material) => {
            // If we do not have a shader assigned, use the default material's shader
            shader = match &user_material.shader {
                Some(id) => match buf.as_shader(&id) {
                    Some(shader) => shader,
                    None => shader,
                },
                None => shader,
            };
            user_material
        }
        None => dm,
    };
    let model = buf.as_model(&renderer.model_id).unwrap();
    let model_matrix = &renderer.matrix;
    // Calculate the mvp matrix
    let mvp_matrix: veclib::Matrix4x4<f32> = camera.projm * camera.viewm * *model_matrix;
    // Pass the MVP and the model matrix to the shader
    let group1 = &buf.as_uniforms(&material.uniforms).unwrap().uniforms;
    let mut group2 = ShaderUniformsGroup::new();
    let settings = ShaderUniformsSettings::new_program_id(shader);
    group2.set_mat44("mvp_matrix", mvp_matrix);
    group2.set_mat44("model_matrix", *model_matrix);
    group2.set_mat44("view_matrix", camera.viewm);
    group2.set_vec3f32("view_pos", camera.position); 
    // Set a default impl uniform
    group2.set_f32("_active_time", renderer.time_alive); 
    group2.set_f32("_time", new_time);
    group2.set_vec2i32("_resolution", resolution);
    group2.set_f32("_delta", new_time);   
    // Combine the two groups
    let mut combined = ShaderUniformsGroup::combine(group1, &group2);

    // Use the custom renderer shader uniforms
    if let Option::Some(group) = &renderer.uniforms {
        let group = &buf.as_uniforms(group).unwrap().uniforms;
        // We might need to combine another time
        combined = ShaderUniformsGroup::combine(&combined, group);
    }
    
    // Update the uniforms
    combined.execute(buf, settings).unwrap();
    
    unsafe {
        // Enable / Disable vertex culling for double sided materials
        if material.flags.contains(MaterialFlags::DOUBLE_SIDED) {
            gl::Disable(gl::CULL_FACE);
        } else {
            gl::Enable(gl::CULL_FACE);
        }

        // Actually draw
        gl::BindVertexArray(model.vertex_array_object);
        gl::BindBuffer(gl::ELEMENT_ARRAY_BUFFER, model.element_buffer_object);
        gl::DrawElements(gl::TRIANGLES, model.element_count as i32, gl::UNSIGNED_INT, null());
    }
}

// Render a renderer using wireframe
fn render_wireframe(buf: &PipelineBuffer, renderer: &RendererGPUObject, camera: &CameraDataGPUObject, ws: &GPUObjectID) {
    let model = buf.as_model(&renderer.model_id).unwrap();
    let model_matrix = &renderer.matrix;
    // Get the wireframe shader
    let ws = buf.as_shader(ws).unwrap();
    let mut group = ShaderUniformsGroup::new();
    let settings = ShaderUniformsSettings::new_program_id(ws);
    // Calculate the mvp matrix
    let mvp_matrix: veclib::Matrix4x4<f32> = camera.projm * camera.viewm * *model_matrix;
    group.set_mat44("mvp_matrix", mvp_matrix);
    group.set_mat44("model_matrix", *model_matrix);
    group.set_mat44("view_matrix", camera.viewm);
    group.execute(buf, settings).unwrap();
    unsafe {
        // Set the wireframe rendering
        gl::PolygonMode(gl::FRONT_AND_BACK, gl::LINE);
        gl::Enable(gl::LINE_SMOOTH);

        gl::BindVertexArray(model.vertex_array_object);
        gl::BindBuffer(gl::ELEMENT_ARRAY_BUFFER, model.element_buffer_object);
        gl::DrawElements(gl::TRIANGLES, model.element_count as i32, gl::UNSIGNED_INT, null());

        // Reset the wireframe settings
        gl::BindTexture(gl::TEXTURE_2D, 0);
        gl::Disable(gl::LINE_SMOOTH);
        gl::PolygonMode(gl::FRONT_AND_BACK, gl::FILL);
    }
}

impl PipelineRenderer {
    // Init the pipeline renderer
    pub fn init(&mut self) {
        println!("Initializing the pipeline renderer...");
        self.window = Window::default();
        // Create the quad model
        use crate::basics::Model;
        use veclib::consts::*;
        let quad = Model {
            vertices: vec![vec3(1.0, -1.0, 0.0), vec3(-1.0, 1.0, 0.0), vec3(-1.0, -1.0, 0.0), vec3(1.0, 1.0, 0.0)],
            normals: vec![veclib::Vector3::ZERO; 4],
            tangents: vec![veclib::Vector4::ZERO; 4],
            uvs: vec![vec2(1.0, 0.0), vec2(0.0, 1.0), vec2(0.0, 0.0), vec2(1.0, 1.0)],
            colors: vec![veclib::Vector3::ZERO; 4],
            triangles: vec![0, 1, 2, 0, 3, 1],
            ..Model::default()
        };
        self.quad_model = pipec::model(quad);
        self.screen_shader = pipec::shader(
            Shader::default()
                .load_shader(vec![
                    "defaults\\shaders\\rendering\\passthrough.vrsh.glsl",
                    "defaults\\shaders\\rendering\\screen.frsh.glsl",
                ])
                .unwrap(),
        );
        // Create a default material
        self.default_material = Some(pipec::material(
            Material::new("Default Material").set_shader(pipec::shader(
                Shader::default()
                    .load_shader(vec!["defaults\\shaders\\rendering\\default.vrsh.glsl", "defaults\\shaders\\rendering\\default.frsh.glsl"])
                    .unwrap(),
            )),
        ));
        println!("Loaded the default material!");
        /* #region Deferred renderer init */
        // Local function for binding a texture to a specific frame buffer attachement

        unsafe {
            gl::GenFramebuffers(1, &mut self.framebuffer);
            gl::BindFramebuffer(gl::FRAMEBUFFER, self.framebuffer);
            let dims = TextureType::Texture2D(self.window.dimensions.x, self.window.dimensions.y);
            // Create the diffuse render texture
            self.diffuse_texture = pipec::texture(Texture::default().set_dimensions(dims).set_format(TextureFormat::RGB32F));
            // Create the normals render texture
            self.normals_texture = pipec::texture(Texture::default().set_dimensions(dims).set_format(TextureFormat::RGB8RS));
            // Create the position render texture
            self.position_texture = pipec::texture(Texture::default().set_dimensions(dims).set_format(TextureFormat::RGB32F));
            // Create the depth render texture
            self.depth_texture = pipec::texture(
                Texture::default()
                    .set_dimensions(dims)
                    .set_format(TextureFormat::DepthComponent32)
                    .set_data_type(DataType::Float32),
            );

            // Now bind the attachememnts
            fn bind_attachement(attachement: u32, texture: &GPUObjectID) {
                // Get the textures from the GPUObjectID
                let buf = crate::BUFFER.lock().unwrap();
                let texture = buf.as_texture(texture).unwrap();
                // Default target, no multisamplind
                let target: u32 = gl::TEXTURE_2D;
                unsafe {
                    gl::BindTexture(target, texture.texture_id);
                    gl::FramebufferTexture2D(gl::FRAMEBUFFER, attachement, target, texture.texture_id, 0);
                }
            }
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
        // Load sky gradient texture
        self.sky_texture = pipec::texturec(
            assets::cachec::acache_l(
                "defaults\\textures\\sky_gradient.png",
                Texture::default().set_wrapping_mode(crate::texture::TextureWrapping::ClampToEdge),
            )
            .unwrap(),
        );

        // Load the wireframe shader
        self.wireframe_shader = pipec::shader(
            Shader::default()
                .load_shader(vec!["defaults\\shaders\\rendering\\default.vrsh.glsl", "defaults\\shaders\\others\\wireframe.frsh.glsl"])
                .unwrap(),
        );
        /* #endregion */
        println!("Successfully initialized the RenderPipeline Renderer!");
    }
    // Pre-render event
    pub fn pre_render(&self) {
        unsafe {
            gl::BindFramebuffer(gl::FRAMEBUFFER, self.framebuffer);
            gl::Clear(gl::COLOR_BUFFER_BIT | gl::DEPTH_BUFFER_BIT);
        }
    }
    // Called each frame, for each renderer that is valid in the pipeline
    pub fn renderer_frame(&self, buf: &PipelineBuffer, camera: &CameraDataGPUObject, new_time: f32, delta: f32) {
        let i = std::time::Instant::now();
        let material = buf.as_material(self.default_material.as_ref().unwrap()).unwrap();
        for renderer in buf.renderers.iter().map(|x| buf.as_renderer(x).unwrap()) {
            // Should we render in wireframe or not?
            if self.wireframe {
                render_wireframe(buf, renderer, camera, &self.wireframe_shader);
            } else {
                render(buf, renderer, camera, material, new_time, delta, self.window.dimensions.into());
            }
        }
    }
    // Post-render event
    pub fn post_render(&self, buf: &PipelineBuffer, camera: &CameraDataGPUObject, window: &mut glfw::Window) {
        // Update the frame stats texture
        //self.frame_stats.update_texture(data.time_manager, &data.entity_manager.entities);
        let dimensions = self.window.dimensions;
        // Render the screen QUAD
        let screen_shader = buf.as_shader(&self.screen_shader).unwrap();
        let settings = ShaderUniformsSettings::new_program_id(screen_shader);
        let mut group = ShaderUniformsGroup::new();
        group.set_vec2i32("resolution", dimensions.into());
        group.set_vec2f32("nf_planes", camera.clip_planes);
        group.set_vec3f32("directional_light_dir", veclib::Vector3::<f32>::ONE.normalized());
        // Textures
        group.set_t2d("diffuse_texture", &self.diffuse_texture, 0);
        group.set_t2d("normals_texture", &self.normals_texture, 1);
        group.set_t2d("position_texture", &self.position_texture, 2);
        group.set_t2d("depth_texture", &self.depth_texture, 3);
        group.set_t2d("default_sky_gradient", &self.sky_texture, 4);
        let vp_m = camera.projm * (veclib::Matrix4x4::from_quaternion(&camera.rotation));
        group.set_mat44("custom_vp_matrix", vp_m);
        // Other params
        group.set_vec3f32("camera_pos", camera.position);
        group.set_i32("debug_view", 0);
        //group.set_t2d("frame_stats", self.frame_stats.texture, 5);
        group.execute(buf, settings).unwrap();

        // Render the screen quad
        let quad_model = buf.as_model(&self.quad_model).unwrap();
        unsafe {
            gl::BindFramebuffer(gl::FRAMEBUFFER, 0);
            gl::Clear(gl::COLOR_BUFFER_BIT | gl::DEPTH_BUFFER_BIT);
            gl::BindVertexArray(quad_model.vertex_array_object);
            gl::BindBuffer(gl::ELEMENT_ARRAY_BUFFER, quad_model.element_buffer_object);
            gl::DrawElements(gl::TRIANGLES, quad_model.element_count as i32, gl::UNSIGNED_INT, null());
            gl::BindVertexArray(0);
            window.swap_buffers();
        }
    }
    // Update window
    pub fn update_window_dimensions(&mut self, window_dimensions: veclib::Vector2<u16>) {
        // Update the size of each texture that is bound to the framebuffer
        let dims = TextureType::Texture2D(window_dimensions.x, window_dimensions.y);
        pipec::task(pipec::RenderTask::TextureUpdateSize(self.diffuse_texture, dims)).wait_execution();
        pipec::task(pipec::RenderTask::TextureUpdateSize(self.depth_texture, dims)).wait_execution();
        pipec::task(pipec::RenderTask::TextureUpdateSize(self.normals_texture, dims)).wait_execution();
        pipec::task(pipec::RenderTask::TextureUpdateSize(self.position_texture, dims)).wait_execution();
        unsafe {
            gl::Viewport(0, 0, window_dimensions.x as i32, window_dimensions.y as i32);
        }
    }
}
