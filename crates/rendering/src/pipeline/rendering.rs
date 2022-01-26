use super::{Pipeline, InternalPipeline};
use crate::{
    basics::{
        material::MaterialFlags,
        model::Model,
        renderer::Renderer,
        shader::{Shader, ShaderSettings},
        texture::{Texture, TextureFormat, TextureType},
        uniforms::{ShaderUniformsGroup, ShaderUniformsSettings},
    },
    object::{ObjectID, PipelineTask},
    pipeline::pipec,
    utils::DataType,
};
use std::ptr::null;

// Pipeline renderer that will render our world
#[derive(Default)]
pub struct PipelineRenderer {
    // The master frame buffer
    framebuffer: u32,

    // Our deferred textures
    diffuse_texture: ObjectID<Texture>,
    emissive_texture: ObjectID<Texture>,
    normals_texture: ObjectID<Texture>,
    position_texture: ObjectID<Texture>,
    depth_texture: ObjectID<Texture>,

    // Screen rendering
    screenshader: ObjectID<Shader>,
    quad_model: ObjectID<Model>,

    // Others
    sky_texture: ObjectID<Texture>,
}
/*
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
*/
impl PipelineRenderer {
    // Render a single renderere
    fn render(&self, pipeline: &Pipeline, renderer: &Renderer) -> Option<()> {
        // Pipeline data
        let camera = &pipeline.camera;
        let material = pipeline.get_material(renderer.material).unwrap();

        // The shader will always be valid
        let _shader = pipeline.get_shader(material.shader).unwrap();
        let model = pipeline.get_model(renderer.model)?;
        let model_matrix = &renderer.matrix;

        // Calculate the mvp matrix
        let mvp_matrix: veclib::Matrix4x4<f32> = camera.projm * camera.viewm * *model_matrix;

        // Pass the MVP and the model matrix to the shader
        let mut group = material.uniforms.clone();
        let settings = ShaderUniformsSettings::new(material.shader);
        group.set_mat44f32("mvp_matrix", mvp_matrix);
        group.set_mat44f32("model_matrix", *model_matrix);
        group.set_mat44f32("view_matrix", camera.viewm);
        group.set_vec3f32("view_pos", camera.position);
        /*
        // Set a default impl uniform
        group.set_f32("_active_time", renderer.time_alive);
        group.set_f32("_time", new_time);
        group.set_vec2i32("_resolution", resolution);
        group.set_f32("_delta", new_time);
        group.set_bool("_fade_anim", renderer.flags.contains(RendererFlags::FADING_ANIMATION));
        */

        // Update the uniforms
        group.execute(pipeline, settings).unwrap();

        unsafe {
            // Enable / Disable vertex culling for double sided materials
            if material.flags.contains(MaterialFlags::DOUBLE_SIDED) {
                gl::Disable(gl::CULL_FACE);
            } else {
                gl::Enable(gl::CULL_FACE);
            }

            // Actually draw
            gl::BindVertexArray(model.1.vertex_array_object);
            gl::BindBuffer(gl::ELEMENT_ARRAY_BUFFER, model.1.element_buffer_object);
            gl::DrawElements(gl::TRIANGLES, model.1.triangle_count as i32, gl::UNSIGNED_INT, null());
        }
        Some(())
    }
    // Initialize this new pipeline renderer
    pub(crate) fn initialize(&mut self, internal: &mut InternalPipeline, pipeline: &mut Pipeline) {
        println!("Initializing the pipeline renderer...");
        // Create the quad model that we will use to render the whole screen
        use veclib::{vec2, vec3};
        let quad = Model {
            vertices: vec![vec3(1.0, -1.0, 0.0), vec3(-1.0, 1.0, 0.0), vec3(-1.0, -1.0, 0.0), vec3(1.0, 1.0, 0.0)],
            normals: vec![veclib::Vector3::ZERO; 4],
            tangents: vec![veclib::Vector4::ZERO; 4],
            uvs: vec![vec2(1.0, 0.0), vec2(0.0, 1.0), vec2(0.0, 0.0), vec2(1.0, 1.0)],
            colors: vec![veclib::Vector3::ZERO; 4],
            triangles: vec![0, 1, 2, 0, 3, 1],
            ..Model::default()
        };
        // Load the quad model
        self.quad_model = pipec::construct(quad, pipeline);
        println!("Quad model {:?}", self.quad_model);

        // Load the screen shader
        let ss = ShaderSettings::default()
            .source("defaults\\shaders\\rendering\\passthrough.vrsh.glsl")
            .source("defaults\\shaders\\rendering\\screen.frsh.glsl");
        self.screenshader = pipec::construct(Shader::new(ss).unwrap(), pipeline);

        /* #region Deferred renderer init */
        // Local function for binding a texture to a specific frame buffer attachement

        unsafe {
            gl::GenFramebuffers(1, &mut self.framebuffer);
            gl::BindFramebuffer(gl::FRAMEBUFFER, self.framebuffer);
            let dims = TextureType::Texture2D(pipeline.window.dimensions.x, pipeline.window.dimensions.y);
            // Create the diffuse render texture
            self.diffuse_texture = pipec::construct(Texture::default().set_dimensions(dims).set_format(TextureFormat::RGB16R), pipeline);
            // Create the emissive render texture
            self.emissive_texture = pipec::construct(Texture::default().set_dimensions(dims).set_format(TextureFormat::RGB32F), pipeline);
            // Create the normals render texture
            self.normals_texture = pipec::construct(Texture::default().set_dimensions(dims).set_format(TextureFormat::RGB8RS), pipeline);
            // Create the position render texture
            self.position_texture = pipec::construct(Texture::default().set_dimensions(dims).set_format(TextureFormat::RGB32F), pipeline);
            // Create the depth render texture
            self.depth_texture = pipec::construct(
                Texture::default()
                    .set_dimensions(dims)
                    .set_format(TextureFormat::DepthComponent32)
                    .set_data_type(DataType::F32),
                pipeline,
            );

            // Now bind the attachememnts
            fn bind_attachement(attachement: u32, texture: &ObjectID<Texture>, pipeline: &Pipeline) -> Option<()> {
                // Get the textures from the GPUObjectID
                let texture = pipeline.get_texture(*texture)?;
                unsafe {
                    gl::BindTexture(texture.target, texture.oid);
                    gl::FramebufferTexture2D(gl::FRAMEBUFFER, attachement, texture.target, texture.oid, 0);
                }
                Some(())
            }
            // Flush
            pipeline.flush(internal, self);
            bind_attachement(gl::COLOR_ATTACHMENT0, &self.diffuse_texture, pipeline).unwrap();
            bind_attachement(gl::COLOR_ATTACHMENT1, &self.emissive_texture, pipeline).unwrap();
            bind_attachement(gl::COLOR_ATTACHMENT2, &self.normals_texture, pipeline).unwrap();
            bind_attachement(gl::COLOR_ATTACHMENT3, &self.position_texture, pipeline).unwrap();
            bind_attachement(gl::DEPTH_ATTACHMENT, &self.depth_texture, pipeline).unwrap();
            let attachements = vec![gl::COLOR_ATTACHMENT0, gl::COLOR_ATTACHMENT1, gl::COLOR_ATTACHMENT2, gl::COLOR_ATTACHMENT3];
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
        self.sky_texture = pipec::construct(
            assets::assetc::dload::<Texture>("defaults\\textures\\sky_gradient.png")
                .unwrap()
                .set_wrapping_mode(crate::basics::texture::TextureWrapping::ClampToEdge),
            pipeline,
        );
        /* #endregion */

        // We must always flush to make sure we execute the tasks internally
        pipeline.flush(internal, self);
        println!("Successfully initialized the RenderPipeline Renderer!");
    }
    // Pre-render event
    pub fn pre_render(&self) {
        unsafe {
            gl::BindFramebuffer(gl::FRAMEBUFFER, self.framebuffer);
            gl::Clear(gl::COLOR_BUFFER_BIT | gl::DEPTH_BUFFER_BIT);
        }
    }
    // Called each frame, to render the world
    pub fn render_frame(&self, pipeline: &Pipeline) {
        let _i = std::time::Instant::now();
        for (_id, renderer) in pipeline.renderers.iter() {
            let _result = self.render(pipeline, renderer);
            //if result.is_none() { println!("Could not render object with ID '{}'!", id); }
        }
    }
    // Post-render event
    pub fn post_render(&self, pipeline: &Pipeline) {
        // Get the pipeline data
        let dimensions = pipeline.window.dimensions;
        let camera = &pipeline.camera;

        // Render the screen quad
        let mut group = ShaderUniformsGroup::new();
        group.set_vec2i32("resolution", dimensions.into());
        group.set_vec2f32("nf_planes", camera.clip_planes);
        group.set_vec3f32("directional_light_dir", veclib::Vector3::<f32>::ONE.normalized());
        // Textures
        group.set_texture("diffuse_texture", self.diffuse_texture, 0);
        group.set_texture("emissive_texture", self.emissive_texture, 1);
        group.set_texture("normals_texture", self.normals_texture, 2);
        group.set_texture("position_texture", self.position_texture, 3);
        group.set_texture("depth_texture", self.depth_texture, 4);
        group.set_texture("default_sky_gradient", self.sky_texture, 5);
        let vp_m = camera.projm * (veclib::Matrix4x4::<f32>::from_quaternion(&camera.rotation));
        group.set_mat44f32("custom_vp_matrix", vp_m);
        // Other params
        group.set_vec3f32("camera_pos", camera.position);
        group.set_vec3f32("camera_dir", camera.rotation.mul_point(veclib::Vector3::Z));

        // Update the uniform settings
        let settings = ShaderUniformsSettings::new(self.screenshader);
        group.execute(pipeline, settings).unwrap();

        // Render the screen quad
        let (_, quad_data) = pipeline.get_model(self.quad_model).unwrap();
        unsafe {
            gl::BindFramebuffer(gl::FRAMEBUFFER, 0);
            gl::Clear(gl::COLOR_BUFFER_BIT | gl::DEPTH_BUFFER_BIT);
            gl::BindVertexArray(quad_data.vertex_array_object);
            gl::BindBuffer(gl::ELEMENT_ARRAY_BUFFER, quad_data.element_buffer_object);
            gl::DrawElements(gl::TRIANGLES, quad_data.triangle_count as i32, gl::UNSIGNED_INT, null());
            gl::BindVertexArray(0);
        }
    }
    // Update window
    pub fn update_window_dimensions(&mut self, window_dimensions: veclib::Vector2<u16>, pipeline: &Pipeline) {
        // Update the size of each texture that is bound to the framebuffer
        let _dims = TextureType::Texture2D(window_dimensions.x, window_dimensions.y);
        pipec::task(PipelineTask::UpdateTextureDimensions(self.diffuse_texture, _dims), pipeline);
        pipec::task(PipelineTask::UpdateTextureDimensions(self.emissive_texture, _dims), pipeline);
        pipec::task(PipelineTask::UpdateTextureDimensions(self.normals_texture, _dims), pipeline);
        pipec::task(PipelineTask::UpdateTextureDimensions(self.position_texture, _dims), pipeline);
        pipec::task(PipelineTask::UpdateTextureDimensions(self.depth_texture, _dims), pipeline);
        unsafe {
            gl::Viewport(0, 0, window_dimensions.x as i32, window_dimensions.y as i32);
        }
    }
}
