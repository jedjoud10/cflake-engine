use crate::{pipec, pipeline::object::*, FrameStats, MaterialFlags, Shader, Texture};
use crate::{texture::*, DataType, Material, Window, GPUObjectID};

use glfw::Context;
use others::SmartList;
use std::collections::HashSet;
use std::ptr::null;

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
    framebuffer: u32,                        // The master frame buffer
    diffuse_texture: GPUObjectID,       // Diffuse texture, can also store HDR values
    normals_texture: GPUObjectID,       // World Normals texture
    position_texture: GPUObjectID,      // World Positions texture
    depth_texture: GPUObjectID,         // Depth texture
    debug_view: u16,                         // OUr currenty debug view mode
    wireframe: bool,                         // Are we rendering in wireframe or not
    quad_model: GPUObjectID,              // The current screen quad model that we are using
    screen_shader: GPUObjectID,          // The current screen quad shader that we are using
    sky_texture: GPUObjectID,           // The sky gradient texture
    wireframe_shader: GPUObjectID,       // The current wireframe shader
    frame_stats: FrameStats,                 // Some frame stats
    pub window: Window,                      // Window
    pub default_material: Option<Material>,  // Self explanatory
    renderer_ids: HashSet<GPUObjectID> // IDs of GPURendererObjects
}

// Render debug primitives
pub fn render_debug_primitives(primitives: Vec<RendererGPUObject>, camera: &CameraDataGPUObject) {
    let _vp_m = camera.projm * camera.viewm;
    for primitive in &primitives {
        render(primitive, camera);
    }
}

// Render a renderer normally
pub fn render(renderer: &RendererGPUObject, camera: &CameraDataGPUObject) {
    let shader = (renderer.1).to_shader().unwrap();
    let material = (renderer.1).to_material().unwrap();
    let model = (renderer.0).to_model().unwrap();
    let model_matrix = &renderer.2;
    // Calculate the mvp matrix
    let mvp_matrix: veclib::Matrix4x4<f32> = camera.projm * camera.viewm * *model_matrix;
    // Pass the MVP and the model matrix to the shader
    let mut group = material.1.clone();
    group.with_shader(shader);
    group.set_mat44("mvp_matrix", mvp_matrix);
    group.set_mat44("model_matrix", *model_matrix);
    group.set_mat44("view_matrix", camera.viewm);
    group.set_vec3f32("view_pos", camera.position);
    group.consume();

    unsafe {
        // Enable / Disable vertex culling for double sided materials
        if material.2.contains(MaterialFlags::DOUBLE_SIDED) {
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
fn render_wireframe(renderer: &RendererGPUObject, camera: &CameraDataGPUObject, ws: &GPUObjectID) {
    let shader = (renderer.1).to_shader().unwrap();
    let material = (renderer.1).to_material().unwrap();
    let model = (renderer.0).to_model().unwrap();
    let model_matrix = &renderer.2;
    let ws = ws.to_shader().unwrap();
    let mut group = ws.new_uniform_group();
    group.with_shader(shader);
    // Calculate the mvp matrix
    let mvp_matrix: veclib::Matrix4x4<f32> = camera.projm * camera.viewm * *model_matrix;
    group.set_mat44("mvp_matrix", mvp_matrix);
    group.set_mat44("model_matrix", *model_matrix);
    group.set_mat44("view_matrix", camera.viewm);
    group.consume();
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
        self.default_material = Some(
            Material::default().set_shader(pipec::shader(
                Shader::default()
                    .load_shader(vec!["defaults\\shaders\\rendering\\default.vrsh.glsl", "defaults\\shaders\\rendering\\default.frsh.glsl"])
                    .unwrap(),
            )),
        );
        println!("Loaded the default material!");
        /* #region Deferred renderer init */
        // Local function for binding a texture to a specific frame buffer attachement
        fn bind_attachement(attachement: u32, texture: &GPUObjectID) {
            // Get the textures from the GPUObjectID
            let texture = texture.to_texture().unwrap();
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
    // Renderers
    pub fn add_renderer(&mut self, renderer: GPUObjectID) {
        self.renderer_ids.insert(renderer);
    }
    pub fn remove_renderer(&mut self, renderer_id: &GPUObjectID) {
        self.renderer_ids.remove(renderer_id);
    }
    // Pre-render event
    pub fn pre_render(&mut self) {
        unsafe {
            gl::BindFramebuffer(gl::FRAMEBUFFER, self.framebuffer);
            gl::Clear(gl::COLOR_BUFFER_BIT | gl::DEPTH_BUFFER_BIT);
        }
    }
    // Called each frame, for each renderer that is valid in the pipeline
    pub fn renderer_frame(&self, camera: &CameraDataGPUObject) {
        for renderer in self.renderer_ids.iter().map(|x| x.to_renderer().unwrap()) {
            // Should we render in wireframe or not?
            if self.wireframe {
                render_wireframe(renderer, camera, &self.wireframe_shader);
            } else {
                render(renderer, camera);
            }
        }
    }
    // Post-render event
    pub fn post_render(&self, camera: &CameraDataGPUObject, window: &mut glfw::Window) {
        // Update the frame stats texture
        //self.frame_stats.update_texture(data.time_manager, &data.entity_manager.entities);
        let dimensions = self.window.dimensions;
        // Render the screen QUAD
        let screen_shader = self.screen_shader.to_shader().unwrap();
        let mut group = screen_shader.new_uniform_group();
        group.set_vec2i32("resolution", dimensions.into());
        group.set_vec2f32("nf_planes", camera.clip_planes);
        group.set_vec3f32("directional_light_dir", veclib::Vector3::<f32>::ONE.normalized());
        // Textures
        group.set_t2d("diffuse_texture", self.diffuse_texture, 0);
        group.set_t2d("normals_texture", self.normals_texture, 1);
        group.set_t2d("position_texture", self.position_texture, 2);
        group.set_t2d("depth_texture", self.depth_texture, 3);
        group.set_t2d("default_sky_gradient", self.sky_texture, 4);
        let vp_m = camera.projm * (veclib::Matrix4x4::from_quaternion(&camera.rotation));
        group.set_mat44("custom_vp_matrix", vp_m);
        // Other params
        group.set_vec3f32("camera_pos", camera.position);
        group.set_i32("debug_view", 0);
        group.set_t2d("frame_stats", self.frame_stats.texture, 5);
        group.consume();

        // Render the screen quad
        let quad_model = self.quad_model.to_model().unwrap();
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
        pipec::task(pipec::RenderTask::TextureUpdateSize(self.diffuse_texture, dims)).wait();
        pipec::task(pipec::RenderTask::TextureUpdateSize(self.depth_texture, dims)).wait();
        pipec::task(pipec::RenderTask::TextureUpdateSize(self.normals_texture, dims)).wait();
        pipec::task(pipec::RenderTask::TextureUpdateSize(self.position_texture, dims)).wait();
        unsafe {
            gl::Viewport(0, 0, window_dimensions.x as i32, window_dimensions.y as i32);
        }
    }
}
