use self::shadow_mapping::ShadowMapping;

use super::{settings::PipelineSettings, InternalPipeline, Pipeline, FrameDebugInfo};
use crate::{
    basics::{
        lights::{LightSource, LightSourceType},
        material::Material,
        model::Model,
        renderer::{Renderer, RendererFlags},
        shader::{Shader, ShaderSettings},
        texture::{Texture, TextureFormat, TextureType},
        uniforms::{ShaderIdentifier, ShaderUniformsGroup, ShaderUniformsSettings},
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
    uniforms: ShaderUniformsGroup,

    // Others
    sky_texture: ObjectID<Texture>,

    shadow_mapping: ShadowMapping,
}
impl PipelineRenderer {
    // Setup uniforms for a specific renderer
    fn configure_uniforms<'a>(&self, pipeline: &'a Pipeline, renderer: &Renderer) -> Option<(&'a Model, &'a Material)> {
        // Pipeline data
        let camera = &pipeline.camera;
        let material = pipeline.materials.get(renderer.material).unwrap();

        // The shader will always be valid
        let shader = pipeline.shaders.get(material.shader).unwrap();
        let model = pipeline.models.get(renderer.model)?;
        let model_matrix = &renderer.matrix;

        // Pass the matrices to the shader
        let mut group = ShaderUniformsGroup::default();
        let settings = ShaderUniformsSettings::new(ShaderIdentifier::OpenGLID(shader.program));
        group.set_mat44f32("project_view_matrix", camera.projm * camera.viewm);
        group.set_mat44f32("model_matrix", *model_matrix);

        // Update the uniforms
        material.uniforms.bind_shader(pipeline, settings);
        material.uniforms.set_uniforms(pipeline, settings);
        if let Some(uniforms) = &renderer.uniforms {
            uniforms.set_uniforms(pipeline, settings);
        }
        group.set_uniforms(pipeline, settings);

        Some((&model, material))
    }
    // Render a single renderer
    fn render(&self, model: &Model, double_sided: bool) {
        unsafe {
            // Enable / Disable vertex culling for double sided materials
            if double_sided {
                gl::Disable(gl::CULL_FACE);
            } else {
                gl::Enable(gl::CULL_FACE);
            }

            // Actually draw
            gl::BindVertexArray(model.vertex_array_object);
            gl::BindBuffer(gl::ELEMENT_ARRAY_BUFFER, model.buffers[0]);
            gl::DrawElements(gl::TRIANGLES, model.triangles.len() as i32, gl::UNSIGNED_INT, null());
        }
    }
    // Initialize this new pipeline renderer
    pub(crate) fn initialize(&mut self, pipeline_settings: PipelineSettings, internal: &mut InternalPipeline, pipeline: &mut Pipeline) {
        println!("Initializing the pipeline renderer...");
        // Create the quad model that we will use to render the whole screen
        use veclib::{vec2, vec3};
        let quad = Model {
            vertices: vec![vec3(1.0, -1.0, 0.0), vec3(-1.0, 1.0, 0.0), vec3(-1.0, -1.0, 0.0), vec3(1.0, 1.0, 0.0)],
            uvs: vec![vec2(255, 0), vec2(0, 255), vec2(0, 0), vec2(255, 255)],
            triangles: vec![0, 1, 2, 0, 3, 1],
            ..Model::default()
        };
        // Load the quad model
        self.quad_model = pipec::construct(pipeline, quad).unwrap();
        println!("Quad model {:?}", self.quad_model);

        // Load the screen shader
        let settings = ShaderSettings::default()
            .source("defaults\\shaders\\rendering\\passthrough.vrsh.glsl")
            .source("defaults\\shaders\\rendering\\screen.frsh.glsl")
            .shader_constant("shadow_bias", pipeline_settings.shadow_bias);
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
                let texture = pipeline.textures.get(*texture)?;
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
        self.shadow_mapping = ShadowMapping::new(self, pipeline_settings.shadow_resolution, internal, pipeline);
        // Load sky gradient texture
        self.sky_texture = pipec::construct(
            pipeline,
            assets::assetc::dload::<Texture>("defaults\\textures\\sky_gradient.png")
                .unwrap()
                .set_wrapping_mode(crate::basics::texture::TextureWrapping::ClampToEdge),
        )
        .unwrap();

        // Also set our one time uniforms
        let mut group = ShaderUniformsGroup::new();
        group.set_texture("diffuse_texture", self.diffuse_texture, 0);
        group.set_texture("emissive_texture", self.emissive_texture, 1);
        group.set_texture("normals_texture", self.normals_texture, 2);
        group.set_texture("position_texture", self.position_texture, 3);
        group.set_texture("depth_texture", self.depth_texture, 4);
        group.set_texture("shadow_map", self.shadow_mapping.depth_texture, 6);
        group.set_texture("default_sky_gradient", self.sky_texture, 5);
        self.uniforms = group;
        /* #endregion */

        // We must always flush to make sure we execute the tasks internally
        pipeline.flush(internal, self);
        println!("Successfully initialized the RenderPipeline Renderer!");
    }
    // Prepare the FBO and clear the buffers
    fn prepare_for_rendering(&self, pipeline: &Pipeline) {
        unsafe {
            gl::Viewport(0, 0, pipeline.window.dimensions.x as i32, pipeline.window.dimensions.y as i32);
            gl::BindFramebuffer(gl::FRAMEBUFFER, self.framebuffer);
            gl::Clear(gl::COLOR_BUFFER_BIT | gl::DEPTH_BUFFER_BIT);
        }
    }
    // Called each frame, to render the world
    pub(crate) fn render_frame(&mut self, pipeline: &Pipeline) -> FrameDebugInfo {
        // Prepare
        let mut debug_info = FrameDebugInfo::default();
        self.prepare_for_rendering(pipeline);
        // Render normally
        self.render_scene(pipeline, &mut debug_info);
        // Then render the scene again so we can render shadows
        self.render_scene_shadow_maps(pipeline, &mut debug_info);
        // Render the deferred quad
        self.render_deferred_quad(pipeline, &mut debug_info);
        debug_info
    }
    // Render the whole scene normally
    fn render_scene(&mut self, pipeline: &Pipeline, debug_info: &mut FrameDebugInfo) {
        for (_, renderer) in pipeline.renderers.iter() {
            // Check if we are visible
            if !renderer.flags.contains(RendererFlags::VISIBLE) {
                continue;
            }
            let result = self.configure_uniforms(pipeline, renderer);
            // The renderer might've failed setting it's uniforms
            if let Some((model, material)) = result {
                self.render(model, material.double_sided);
                debug_info.draw_calls += 1;
                debug_info.triangles += model.triangles.len() as u64;
                debug_info.vertices += model.vertices.len() as u64;
            }
        }
    }
    // Render the scene's shadow maps
    fn render_scene_shadow_maps(&mut self, pipeline: &Pipeline, debug_info: &mut FrameDebugInfo) {
        // Check if shadows are even enabled in the first place
        if !self.shadow_mapping.enabled {
            return;
        }

        self.shadow_mapping.bind_fbo();
        let directional_light_source = pipeline.light_sources.get(pipeline.defaults.as_ref().unwrap().sun);
        if let Some(light) = directional_light_source {
            self.shadow_mapping.update_view_matrix(*light._type.as_directional().unwrap());
        }
        for (_, renderer) in pipeline.renderers.iter() {
            // Check if we should cast shadows
            if !renderer.flags.contains(RendererFlags::SHADOW_CASTER) {
                continue;
            }

            let result = self.shadow_mapping.configure_uniforms(pipeline, renderer);
            // The renderer might've failed setting it's uniforms
            if let Some(model) = result {
                self.render(model, false);
                debug_info.shadow_draw_calls += 1;
            }
        }
        unsafe {
            gl::CullFace(gl::BACK);
        }
    }
    // Render the deferred quad and do all lighting calculations inside it's fragment shader
    fn render_deferred_quad(&mut self, pipeline: &Pipeline, debug_info: &mut FrameDebugInfo) {
        unsafe {
            gl::Viewport(0, 0, pipeline.window.dimensions.x as i32, pipeline.window.dimensions.y as i32);
        }
        // Get the pipeline data
        let camera = &pipeline.camera;

        // Render the screen quad
        self.uniforms.set_vec2f32("nf_planes", camera.clip_planes);
        // The first light source is always the directional light source
        let default_light_source = LightSource::new(LightSourceType::Directional {
            quat: veclib::Quaternion::<f32>::IDENTITY,
        });
        let light = pipeline.light_sources.get(pipeline.defaults.as_ref().unwrap().sun).unwrap_or(&default_light_source);
        let directional = light._type.as_directional().unwrap();
        self.uniforms.set_vec3f32("directional_light_dir", directional.mul_point(veclib::Vector3::Z));
        self.uniforms.set_f32("directional_light_strength", light.strength);
        self.uniforms.set_mat44f32("lightspace_matrix", self.shadow_mapping.lightspace_matrix);
        let pr_m = camera.projm * (veclib::Matrix4x4::<f32>::from_quaternion(&camera.rotation));
        self.uniforms.set_mat44f32("projection_rotation_matrix", pr_m);
        // Other params
        self.uniforms.set_vec3f32("camera_pos", camera.position);
        self.uniforms.set_vec3f32("camera_dir", camera.rotation.mul_point(veclib::Vector3::Z));

        // Update the uniform settings
        let settings = ShaderUniformsSettings::new(ShaderIdentifier::ObjectID(self.screenshader));
        self.uniforms.bind_shader(pipeline, settings);
        self.uniforms.set_uniforms(pipeline, settings).unwrap();

        // Render the screen quad
        let quad_model = pipeline.models.get(self.quad_model).unwrap();
        unsafe {
            gl::BindFramebuffer(gl::FRAMEBUFFER, 0);
            gl::Clear(gl::COLOR_BUFFER_BIT | gl::DEPTH_BUFFER_BIT);
            self.render(quad_model, false);
            gl::BindVertexArray(0);
        }
    }
    // Update window
    pub(crate) fn update_window_dimensions(&mut self, window_dimensions: veclib::Vector2<u16>, pipeline: &mut Pipeline) {
        // Update the size of each texture that is bound to the framebuffer
        let dims = TextureType::Texture2D(window_dimensions.x, window_dimensions.y);
        let diffuse_texture = pipeline.textures.get_mut(self.diffuse_texture).unwrap();
        diffuse_texture.update_size(dims).unwrap();
        let emissive_texture = pipeline.textures.get_mut(self.emissive_texture).unwrap();
        emissive_texture.update_size(dims).unwrap();
        let normals_texture = pipeline.textures.get_mut(self.normals_texture).unwrap();
        normals_texture.update_size(dims).unwrap();
        let position_texture = pipeline.textures.get_mut(self.position_texture).unwrap();
        position_texture.update_size(dims).unwrap();
        let depth_texture = pipeline.textures.get_mut(self.depth_texture).unwrap();
        depth_texture.update_size(dims).unwrap();
    }
}
