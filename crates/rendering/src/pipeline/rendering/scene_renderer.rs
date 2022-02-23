use super::super::{settings::PipelineSettings, FrameDebugInfo, InternalPipeline, Pipeline};
use super::postprocessing::PostProcessing;
use super::{error::*, shadow_mapping::ShadowMapping};
use crate::pipeline::camera::Camera;
use crate::{
    basics::{
        lights::{LightSource, LightSourceType},
        material::Material,
        mesh::{Mesh, Vertices},
        renderer::{Renderer, RendererFlags},
        shader::{Shader, ShaderSettings},
        texture::{Texture, TextureFormat, TextureType},
        uniforms::{ShaderIDType, ShaderUniformsSettings, Uniforms},
    },
    object::ObjectID,
    pipeline::pipec,
    utils::DataType,
};
use gl::types::GLuint;
use std::ptr::null;

// Pipeline renderer that will render our world
#[derive(Default)]
pub struct SceneRenderer {
    framebuffer: GLuint,

    // Our deferred textures
    pub(crate) diffuse_texture: ObjectID<Texture>,
    pub(crate) emissive_texture: ObjectID<Texture>,
    pub(crate) normals_texture: ObjectID<Texture>,
    pub(crate) position_texture: ObjectID<Texture>,
    pub(crate) depth_texture: ObjectID<Texture>,

    // Screen rendering
    lighting_pass_screenshader: ObjectID<Shader>,
    quad_model: ObjectID<Mesh>,

    // Others
    sky_gradient: ObjectID<Texture>,
    shadow_mapping: Option<ShadowMapping>,
    postprocessing: PostProcessing,
}
impl SceneRenderer {
    // Initialize this new pipeline renderer
    pub(crate) fn initialize(
        &mut self,
        pipeline_settings: PipelineSettings,
        internal: &mut InternalPipeline,
        pipeline: &mut Pipeline,
    ) {
        log::info!("Initializing the pipeline renderer...");
        // Create the quad mesh that we will use to render the whole screen
        use veclib::{vec2, vec3};
        let quad = Mesh {
            vertices: Vertices {
                positions: vec![
                    vec3(1.0, -1.0, 0.0),
                    vec3(-1.0, 1.0, 0.0),
                    vec3(-1.0, -1.0, 0.0),
                    vec3(1.0, 1.0, 0.0),
                ],
                uvs: vec![vec2(255, 0), vec2(0, 255), vec2(0, 0), vec2(255, 255)],
                ..Default::default()
            },
            triangles: vec![0, 1, 2, 0, 3, 1],
            ..Default::default()
        };
        // Load the quad mesh
        self.quad_model = pipec::construct(pipeline, quad).unwrap();
        log::info!("Quad mesh {:?}", self.quad_model);

        // Load the lighting pass shader
        let settings = ShaderSettings::default()
            .source("defaults/shaders/rendering/passthrough.vrsh.glsl")
            .source("defaults/shaders/rendering/lighting_pass.frsh.glsl")
            .shader_constant("shadow_bias", pipeline_settings.shadow_bias);
        self.lighting_pass_screenshader =
            pipec::construct(pipeline, Shader::new(settings).unwrap()).unwrap();
        /* #region Deferred renderer init */
        // Local function for binding a texture to a specific frame buffer attachement
        let dims =
            TextureType::Texture2D(pipeline.window.dimensions.x, pipeline.window.dimensions.y);
        unsafe {
            gl::GenFramebuffers(1, &mut self.framebuffer);
            gl::BindFramebuffer(gl::FRAMEBUFFER, self.framebuffer);
            // Create the diffuse render texture
            self.diffuse_texture = pipec::construct(
                pipeline,
                Texture::default()
                    .with_dimensions(dims)
                    .with_format(TextureFormat::RGB8R),
            )
            .unwrap();
            // Create the emissive render texture
            self.emissive_texture = pipec::construct(
                pipeline,
                Texture::default()
                    .with_dimensions(dims)
                    .with_format(TextureFormat::RGB32F),
            )
            .unwrap();
            // Create the normals render texture
            self.normals_texture = pipec::construct(
                pipeline,
                Texture::default()
                    .with_dimensions(dims)
                    .with_format(TextureFormat::RGB8RS),
            )
            .unwrap();
            // Create the position render texture
            self.position_texture = pipec::construct(
                pipeline,
                Texture::default()
                    .with_dimensions(dims)
                    .with_format(TextureFormat::RGB32F),
            )
            .unwrap();
            // Create the depth render texture
            self.depth_texture = pipec::construct(
                pipeline,
                Texture::default()
                    .with_dimensions(dims)
                    .with_format(TextureFormat::DepthComponent32)
                    .with_data_type(DataType::F32),
            )
            .unwrap();

            // Now bind the attachememnts
            fn bind_attachement(
                attachement: u32,
                texture: &ObjectID<Texture>,
                pipeline: &Pipeline,
            ) -> Option<()> {
                // Get the textures from the GPUObjectID
                let texture = pipeline.textures.get(*texture)?;
                unsafe {
                    gl::BindTexture(texture.target, texture.oid);
                    gl::FramebufferTexture2D(
                        gl::FRAMEBUFFER,
                        attachement,
                        texture.target,
                        texture.oid,
                        0,
                    );
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
            let attachements = vec![
                gl::COLOR_ATTACHMENT0,
                gl::COLOR_ATTACHMENT1,
                gl::COLOR_ATTACHMENT2,
                gl::COLOR_ATTACHMENT3,
            ];
            gl::DrawBuffers(
                attachements.len() as i32,
                attachements.as_ptr() as *const u32,
            );

            // Check frame buffer state
            if gl::CheckFramebufferStatus(gl::FRAMEBUFFER) != gl::FRAMEBUFFER_COMPLETE {
                panic!(
                    "Framebuffer has failed initialization! Error: '{:#x}'",
                    gl::CheckFramebufferStatus(gl::FRAMEBUFFER)
                );
            }

            // Unbind
            gl::BindFramebuffer(gl::FRAMEBUFFER, 0);
        }
        /* #endregion */
        /* #region Others */
        self.shadow_mapping = if pipeline_settings.shadow_resolution != 0 {
            Some(ShadowMapping::new(
                self,
                pipeline_settings.shadow_resolution,
                internal,
                pipeline,
            ))
        } else {
            None
        };
        self.postprocessing = PostProcessing::new(self, internal, pipeline, dims);
        // Load sky gradient texture
        self.sky_gradient = pipec::construct(
            pipeline,
            assets::assetc::load::<Texture>("defaults/textures/sky_gradient.png")
                .unwrap()
                .with_wrapping_mode(crate::basics::texture::TextureWrapping::ClampToEdge),
        )
        .unwrap();
        /* #endregion */

        // We must always flush to make sure we execute the tasks internally
        pipeline.flush(internal, self);
        log::info!("Successfully initialized the RenderPipeline Renderer!");
    }
    // Get the fallback, default texture IDs in case the provided ones are not valid
    fn get_diffuse_map(pipeline: &Pipeline, material: &Material) -> ObjectID<Texture> {
        material
            .diffuse_map
            .get()
            .map_or_else(|| pipeline.defaults.as_ref().unwrap().white, ObjectID::new)
    }
    fn get_normal_map(pipeline: &Pipeline, material: &Material) -> ObjectID<Texture> {
        material.normal_map.get().map_or_else(
            || pipeline.defaults.as_ref().unwrap().normals_tex,
            ObjectID::new,
        )
    }
    fn get_emissive_map(pipeline: &Pipeline, material: &Material) -> ObjectID<Texture> {
        material
            .emissive_map
            .get()
            .map_or_else(|| pipeline.defaults.as_ref().unwrap().black, ObjectID::new)
    }
    // Setup uniforms for a specific renderer
    fn configure_uniforms<'a>(
        &self,
        pipeline: &'a Pipeline,
        renderer: &Renderer,
    ) -> Result<&'a Mesh, RenderingError> {
        // Pipeline data
        let camera = &pipeline.camera;
        let material = pipeline.materials.get(renderer.material);
        // Load the default material if we don't have a valid one
        let material = material
            .or_else(|| {
                let id = pipeline.defaults.as_ref().unwrap().material;
                Some(pipeline.materials.get(id).unwrap())
            })
            .unwrap();

        // The shader will always be valid
        let shader = pipeline
            .shaders
            .get(material.shader)
            .ok_or(RenderingError)?;
        let mesh = pipeline.meshes.get(renderer.mesh).ok_or(RenderingError)?;
        let model_matrix = &renderer.matrix;
        let settings = ShaderUniformsSettings::new(ShaderIDType::OpenGLID(shader.program));
        let uniforms = Uniforms::new(&settings, pipeline);
        // Bind first
        uniforms.bind_shader();
        // Then set the uniforms
        uniforms.set_mat44f32("project_view_matrix", camera.projm * camera.viewm);
        uniforms.set_mat44f32("model_matrix", *model_matrix);
        // Optional
        material.uniforms.execute(&uniforms);
        renderer.uniforms.execute(&uniforms);
        // Textures might be not valid, so we fallback to the default ones just in case
        uniforms.set_texture("diffuse_tex", Self::get_diffuse_map(pipeline, material), 0);
        uniforms.set_texture("normals_tex", Self::get_normal_map(pipeline, material), 1);
        uniforms.set_texture(
            "emissive_tex",
            Self::get_emissive_map(pipeline, material),
            2,
        );
        uniforms.set_vec3f32("tint", material.tint);
        uniforms.set_f32("normals_strength", material.normal_map_strength);
        uniforms.set_f32("emissive_strength", material.emissive_map_strength);
        uniforms.set_vec2f32("uv_scale", material.uv_scale);

        Ok(mesh)
    }
    // Render a single renderer
    fn render(mesh: &Mesh) {
        unsafe {
            // Actually draw the mesh
            gl::BindVertexArray(mesh.vertex_array_object);
            gl::BindBuffer(gl::ELEMENT_ARRAY_BUFFER, mesh.buffers[0]);
            gl::DrawElements(
                gl::TRIANGLES,
                mesh.tris_count as i32 * 3,
                gl::UNSIGNED_INT,
                null(),
            );
        }
    }
    // Prepare the FBO and clear the buffers
    fn prepare_for_rendering(&self, pipeline: &Pipeline) {
        unsafe {
            gl::Viewport(
                0,
                0,
                pipeline.window.dimensions.x as i32,
                pipeline.window.dimensions.y as i32,
            );
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
        self.shadow_mapping.as_mut().map(|shadow_mapping| {
            Self::render_scene_shadow_maps(shadow_mapping, pipeline, &mut debug_info);
        });
        // Render the deferred quad and the post processing quad
        self.draw_deferred_quad(pipeline);
        self.draw_postprocessing_quad(pipeline);
        debug_info
    }
    // Render the whole scene normally
    fn render_scene(&mut self, pipeline: &Pipeline, debug_info: &mut FrameDebugInfo) {
        let _camera = &pipeline.camera;
        for (_, renderer) in pipeline.renderers.iter() {
            // Check if we are visible
            if !renderer.flags.contains(RendererFlags::VISIBLE) {
                continue;
            }
            let result = self.configure_uniforms(pipeline, renderer);
            // The renderer might've failed setting it's uniforms
            if let Ok(mesh) = result {
                Self::render(mesh);
                debug_info.draw_calls += 1;
                debug_info.triangles += mesh.tris_count as u64;
                debug_info.vertices += mesh.vert_count as u64;
            }
        }
    }
    // Render the scene's shadow maps
    fn render_scene_shadow_maps(
        shadow_mapping: &mut ShadowMapping,
        pipeline: &Pipeline,
        debug_info: &mut FrameDebugInfo,
    ) {
        // Change the cull face for better depth
        unsafe {
            gl::CullFace(gl::FRONT);
        }
        // Bind VBO and set light source
        shadow_mapping.bind_fbo(pipeline);
        let directional_light_source = pipeline
            .light_sources
            .get(pipeline.defaults.as_ref().unwrap().sun);
        if let Some(light) = directional_light_source {
            shadow_mapping.update_view_matrix(*light._type.as_directional().unwrap());
        }

        // Draw the renderers
        for (_, renderer) in pipeline.renderers.iter() {
            // Check if we should cast shadows
            if !renderer.flags.contains(RendererFlags::SHADOW_CASTER) {
                continue;
            }
            let result = shadow_mapping.configure_uniforms(pipeline, renderer);
            // The renderer might've failed setting it's uniforms
            if let Ok(mesh) = result {
                Self::render(mesh);
                debug_info.shadow_draw_calls += 1;
            }
        }
        // Reset
        unsafe {
            gl::CullFace(gl::BACK);
        }
    }
    // Draw the deferred quad and do all lighting calculations inside it's fragment shader
    fn draw_deferred_quad(&mut self, pipeline: &Pipeline) {
        unsafe {
            gl::Viewport(
                0,
                0,
                pipeline.window.dimensions.x as i32,
                pipeline.window.dimensions.y as i32,
            );
        }
        // Get the pipeline data
        let camera = &pipeline.camera;

        // New uniforms
        let settings =
            ShaderUniformsSettings::new(ShaderIDType::ObjectID(self.lighting_pass_screenshader));
        let uniforms = Uniforms::new(&settings, pipeline);
        self.bind_screen_quad_uniforms(uniforms, pipeline, camera);

        // Draw the quad
        let quad_model = pipeline.meshes.get(self.quad_model).unwrap();
        unsafe {
            // Draw to the postprocessing's framebuffer instead
            gl::BindFramebuffer(gl::FRAMEBUFFER, self.postprocessing.framebuffer);
            gl::Disable(gl::DEPTH_TEST);
            Self::render(quad_model);
        }
    }
    // Draw the postprocessing quad and render the color texture
    fn draw_postprocessing_quad(&mut self, pipeline: &Pipeline) {
        self.postprocessing.bind_fbo(pipeline, self);

        // Draw the quad
        let quad_model = pipeline.meshes.get(self.quad_model).unwrap();
        unsafe {
            // Draw to the postprocessing's framebuffer instead
            gl::BindFramebuffer(gl::FRAMEBUFFER, 0);
            Self::render(quad_model);
            gl::BindVertexArray(0);
            gl::Enable(gl::DEPTH_TEST);
        }
    }
    // Name is pretty self explanatory
    fn bind_screen_quad_uniforms(
        &mut self,
        uniforms: Uniforms,
        pipeline: &Pipeline,
        camera: &Camera,
    ) {
        uniforms.bind_shader();
        // The first directional light source is always the sun's light source
        let default = LightSource::new(LightSourceType::Directional {
            quat: veclib::Quaternion::<f32>::IDENTITY,
        });
        let light = pipeline
            .light_sources
            .get(pipeline.defaults.as_ref().unwrap().sun)
            .unwrap_or(&default);
        let directional = light._type.as_directional().unwrap();
        uniforms.set_vec3f32("sunlight_dir", directional.mul_point(veclib::Vector3::Z));
        uniforms.set_f32("sunlight_strength", light.strength);
        uniforms.set_mat44f32(
            "lightspace_matrix",
            self.shadow_mapping
                .as_ref()
                .map_or(veclib::Matrix4x4::default(), |shadow_mapping| {
                    shadow_mapping.lightspace_matrix
                }),
        );
        let pr_m = camera.projm * (veclib::Matrix4x4::<f32>::from_quaternion(&camera.rotation));
        uniforms.set_mat44f32("pr_matrix", pr_m);
        uniforms.set_mat44f32("pv_matrix", camera.projm * camera.viewm);
        uniforms.set_vec2f32("nf_planes", camera.clip_planes);
        // Also gotta set the one time uniforms
        uniforms.set_texture("diffuse_texture", self.diffuse_texture, 0);
        uniforms.set_texture("emissive_texture", self.emissive_texture, 1);
        uniforms.set_texture("normals_texture", self.normals_texture, 2);
        uniforms.set_texture("position_texture", self.position_texture, 3);
        uniforms.set_texture("depth_texture", self.depth_texture, 4);
        uniforms.set_texture("sky_gradient", self.sky_gradient, 5);

        // If we have shadow mapping disabled we must use the default white texture
        let shadow_mapping_texture = self.shadow_mapping.as_ref().map_or(
            pipeline.defaults.as_ref().unwrap().white,
            |shadow_mapping| shadow_mapping.depth_texture,
        );
        uniforms.set_texture("shadow_map", shadow_mapping_texture, 6);
        uniforms.set_bool("shadows_enabled", self.shadow_mapping.is_some());
    }
    // Update window
    pub(crate) fn update_window_dimensions(
        &mut self,
        window_dimensions: veclib::Vector2<u16>,
        pipeline: &mut Pipeline,
    ) {
        // Update the size of each texture that is bound to the framebuffer
        let dims = TextureType::Texture2D(window_dimensions.x, window_dimensions.y);
        let diffuse_texture = pipeline.textures.get_mut(self.diffuse_texture).unwrap();
        diffuse_texture.update_size_fill(dims, Vec::new()).unwrap();
        let emissive_texture = pipeline.textures.get_mut(self.emissive_texture).unwrap();
        emissive_texture.update_size_fill(dims, Vec::new()).unwrap();
        let normals_texture = pipeline.textures.get_mut(self.normals_texture).unwrap();
        normals_texture.update_size_fill(dims, Vec::new()).unwrap();
        let position_texture = pipeline.textures.get_mut(self.position_texture).unwrap();
        position_texture.update_size_fill(dims, Vec::new()).unwrap();
        let depth_texture = pipeline.textures.get_mut(self.depth_texture).unwrap();
        depth_texture.update_size_fill(dims, Vec::new()).unwrap();

        // Don't forget the postprocessing color texture
        self.postprocessing.resize_texture(dims, pipeline);
    }
}
