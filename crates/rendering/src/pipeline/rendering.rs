use self::shadow_mapping::ShadowMapping;

use super::{InternalPipeline, Pipeline};
use crate::{
    basics::{
        model::{Model, ModelBuffers},
        renderer::Renderer,
        shader::{Shader, ShaderSettings},
        texture::{Texture, TextureFormat, TextureType},
        uniforms::{ShaderIdentifier, ShaderUniformsGroup, ShaderUniformsSettings}, material::Material,
    },
    object::ObjectID,
    pipeline::pipec,
    utils::DataType,
};
use std::ptr::null;
mod shadow_mapping;

// Pipeline renderer that will render our world
#[derive(Default)]
pub struct PipelineRenderer {
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

    shadow_mapping: ShadowMapping,
}
impl PipelineRenderer {
    // Setup uniforms for a specific renderer
    fn configure_uniforms<'a>(&self, pipeline: &'a Pipeline, renderer: &Renderer) -> Option<(&'a ModelBuffers, usize, &'a Material)> {
        // Pipeline data
        let camera = &pipeline.camera;
        let material = pipeline.get_material(renderer.material).unwrap();

        // The shader will always be valid
        let shader = pipeline.get_shader(material.shader).unwrap();
        let model = pipeline.get_model(renderer.model)?;
        let model_matrix = &renderer.matrix;

        // Calculate the mvp matrix
        let mvp_matrix: veclib::Matrix4x4<f32> = camera.projm * camera.viewm * *model_matrix;

        // Pass the MVP and the model matrix to the shader
        let mut group = ShaderUniformsGroup::combine(material.uniforms.clone(), renderer.uniforms.clone().unwrap_or_default());
        let settings = ShaderUniformsSettings::new(ShaderIdentifier::OpenGLID(shader.program));
        group.set_mat44f32("mvp_matrix", mvp_matrix);
        group.set_mat44f32("model_matrix", *model_matrix);
        group.set_mat44f32("view_matrix", camera.viewm);
        group.set_vec3f32("view_pos", camera.position);

        // Update the uniforms
        group.execute(pipeline, settings).unwrap();
        Some((&model.1, model.0.triangles.len(), material))
    }
    // Render a single renderer
    fn render(&self, buffers: &ModelBuffers, triangle_count: usize, double_sided: bool) -> Option<()> {
        unsafe {
            // Enable / Disable vertex culling for double sided materials
            if double_sided {
                gl::Disable(gl::CULL_FACE);
            } else {
                gl::Enable(gl::CULL_FACE);
            }

            // Actually draw
            gl::BindVertexArray(buffers.vertex_array_object);
            gl::BindBuffer(gl::ELEMENT_ARRAY_BUFFER, buffers.element_buffer_object);
            gl::DrawElements(gl::TRIANGLES, triangle_count as i32, gl::UNSIGNED_INT, null());
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
        self.quad_model = pipec::construct(pipeline, quad).unwrap();
        println!("Quad model {:?}", self.quad_model);

        // Load the screen shader
        let settings = ShaderSettings::default()
            .source("defaults\\shaders\\rendering\\passthrough.vrsh.glsl")
            .source("defaults\\shaders\\rendering\\screen.frsh.glsl");
        self.screenshader = pipec::construct(pipeline, Shader::new(settings).unwrap()).unwrap();

        /* #region Deferred renderer init */
        // Local function for binding a texture to a specific frame buffer attachement

        unsafe {
            gl::GenFramebuffers(1, &mut self.framebuffer);
            gl::BindFramebuffer(gl::FRAMEBUFFER, self.framebuffer);
            let dims = TextureType::Texture2D(pipeline.window.dimensions.x, pipeline.window.dimensions.y);
            // Create the diffuse render texture
            self.diffuse_texture = pipec::construct(pipeline, Texture::default().set_dimensions(dims).set_format(TextureFormat::RGB16R)).unwrap();
            // Create the emissive render texture
            self.emissive_texture = pipec::construct(pipeline, Texture::default().set_dimensions(dims).set_format(TextureFormat::RGB32F)).unwrap();
            // Create the normals render texture
            self.normals_texture = pipec::construct(pipeline, Texture::default().set_dimensions(dims).set_format(TextureFormat::RGB8RS)).unwrap();
            // Create the position render texture
            self.position_texture = pipec::construct(pipeline, Texture::default().set_dimensions(dims).set_format(TextureFormat::RGB32F)).unwrap();
            // Create the depth render texture
            self.depth_texture = pipec::construct(
                pipeline,
                Texture::default()
                    .set_dimensions(dims)
                    .set_format(TextureFormat::DepthComponent32)
                    .set_data_type(DataType::F32),
            )
            .unwrap();

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
        /* #region Others */
        self.shadow_mapping = ShadowMapping::new(self, internal, pipeline);
        /* #endregion */
        /* #region Actual pipeline renderer shit */
        // Load sky gradient texture
        self.sky_texture = pipec::construct(
            pipeline,
            assets::assetc::dload::<Texture>("defaults\\textures\\sky_gradient.png")
                .unwrap()
                .set_wrapping_mode(crate::basics::texture::TextureWrapping::ClampToEdge),
        )
        .unwrap();
        /* #endregion */

        // We must always flush to make sure we execute the tasks internally
        pipeline.flush(internal, self);
        println!("Successfully initialized the RenderPipeline Renderer!");
    }
    // Prepare the FBO and clear the buffers
    fn prepare_for_rendering(&self) {
        unsafe {
            gl::BindFramebuffer(gl::FRAMEBUFFER, self.framebuffer);
            gl::Clear(gl::COLOR_BUFFER_BIT | gl::DEPTH_BUFFER_BIT);
        }
    }
    // Called each frame, to render the world
    pub(crate) fn render_frame(&mut self, pipeline: &Pipeline) {
        // Prepare
        self.prepare_for_rendering();
        // Render normally
        self.render_scene(pipeline);
        // Then render the scene again so we can render shadows
        self.render_scene_shadow_maps(pipeline);
        // Render the deferred quad
        self.render_deferred_quad(pipeline);
    }
    // Render the scene's shadow maps
    fn render_scene_shadow_maps(&mut self, pipeline: &Pipeline) {
        self.shadow_mapping.bind_fbo();
        let directional_light_source = pipeline.get_light_source(pipeline.defaults.as_ref().unwrap().sun);
        if let Some(light) = directional_light_source {
            self.shadow_mapping.update_view_matrix(*light._type.as_directional().unwrap());
        }
        for (_, renderer) in pipeline.renderers.iter() {
            let result = self.shadow_mapping.configure_uniforms(pipeline, renderer);
            // The renderer might've failed setting it's uniforms
            if let Some((buffers, triangle_count)) = result {
                self.render(buffers, triangle_count, false);
            }
        }
    }
    // Render the whole scene normally
    fn render_scene(&mut self, pipeline: &Pipeline) {
        for (_, renderer) in pipeline.renderers.iter() {
            let result = self.configure_uniforms(pipeline, renderer);
            // The renderer might've failed setting it's uniforms
            if let Some((buffers, triangle_count, material)) = result {
                self.render(buffers, triangle_count, material.double_sided);
            }
        }
    }
    // Render the deferred quad and do all lighting calculations inside it's fragment shader
    fn render_deferred_quad(&mut self, pipeline: &Pipeline) {
        // Get the pipeline data
        let dimensions = pipeline.window.dimensions;
        unsafe {
            gl::Viewport(0, 0, dimensions.x as i32, dimensions.y as i32);
        }
        let camera = &pipeline.camera;

        // Render the screen quad
        let mut group = ShaderUniformsGroup::new();
        group.set_vec2i32("resolution", dimensions.into());
        group.set_vec2f32("nf_planes", camera.clip_planes);
        // The first light source is always the directional light source
        let directional_light_source = pipeline.get_light_source(pipeline.defaults.as_ref().unwrap().sun);
        if let Some(light) = directional_light_source {
            let directional = light._type.as_directional().unwrap();
            group.set_vec3f32("directional_light_dir", directional.mul_point(veclib::Vector3::Z));
            group.set_f32("directional_light_strength", light.strength);
            group.set_mat44f32("lightspace_matrix", self.shadow_mapping.lightspace_matrix);
        } else {
            // We don't have a directional light, so we must set the default values
            group.set_vec3f32("directional_light_dir", veclib::Vector3::ZERO);
            group.set_f32("directional_light_strength", 0.0);
            group.set_mat44f32("lightspace_matrix", veclib::Matrix4x4::IDENTITY);
        }
        // Textures
        group.set_texture("diffuse_texture", self.diffuse_texture, 0);
        group.set_texture("emissive_texture", self.emissive_texture, 1);
        group.set_texture("normals_texture", self.normals_texture, 2);
        group.set_texture("position_texture", self.position_texture, 3);
        group.set_texture("depth_texture", self.depth_texture, 4);
        group.set_texture("default_sky_gradient", self.sky_texture, 5);
        group.set_texture("shadow_map", self.shadow_mapping.depth_texture, 6);
        let vp_m = camera.projm * (veclib::Matrix4x4::<f32>::from_quaternion(&camera.rotation));
        group.set_mat44f32("custom_vp_matrix", vp_m);
        // Other params
        group.set_vec3f32("camera_pos", camera.position);
        group.set_vec3f32("camera_dir", camera.rotation.mul_point(veclib::Vector3::Z));

        // Update the uniform settings
        let settings = ShaderUniformsSettings::new(ShaderIdentifier::ObjectID(self.screenshader));
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
    pub(crate) fn update_window_dimensions(&mut self, window_dimensions: veclib::Vector2<u16>, pipeline: &mut Pipeline) {
        // Update the size of each texture that is bound to the framebuffer
        let dims = TextureType::Texture2D(window_dimensions.x, window_dimensions.y);
        let diffuse_texture = pipeline.get_texture_mut(self.diffuse_texture).unwrap();
        diffuse_texture.update_size(dims).unwrap();
        let emissive_texture = pipeline.get_texture_mut(self.emissive_texture).unwrap();
        emissive_texture.update_size(dims).unwrap();
        let normals_texture = pipeline.get_texture_mut(self.normals_texture).unwrap();
        normals_texture.update_size(dims).unwrap();
        let position_texture = pipeline.get_texture_mut(self.position_texture).unwrap();
        position_texture.update_size(dims).unwrap();
        let depth_texture = pipeline.get_texture_mut(self.depth_texture).unwrap();
        depth_texture.update_size(dims).unwrap();
    }
}
