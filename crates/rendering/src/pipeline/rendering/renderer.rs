use super::{common, RenderingSettings, ShadowMapping};
use crate::{
    basics::{
        mesh::{Mesh, Vertices},
        shader::{Directive, Shader, ShaderInitSettings},
        texture::{Texture, TextureBuilder, TextureDimensions, TextureFormat, TextureWrapping},
    },
    pipeline::{Handle, Pipeline},
    utils::DataType,
};
use assets::assetc;
use gl::types::GLuint;

// Scene renderer that will render our world using deferred rendering
// TODO: Document
#[derive(Default)]
pub struct SceneRenderer {
    // Frame buffer
    framebuffer: GLuint,

    // Our deferred textures
    /*
    diffuse_texture: Handle<Texture>,
    emissive_texture: Handle<Texture>,
    normals_texture: Handle<Texture>,
    position_texture: Handle<Texture>,
    depth_texture: Handle<Texture>,
    */
    textures: [Handle<Texture>; 5],

    // Screen rendering
    lighting_pass: Handle<Shader>,
    quad: Handle<Mesh>,

    // Others
    sky_gradient: Handle<Texture>,
    shadow_mapping: Option<ShadowMapping>,
}

impl SceneRenderer {
    // Initialize a new scene renderer
    pub(crate) unsafe fn new(pipeline: &mut Pipeline) -> Self {
        println!("Initializing the scene renderer...");
        /* #region Quad */
        // Create the quad mesh that we will use to render the whole screen
        use veclib::{vec2, vec3};
        let quad = Mesh::new(
            Vertices {
                positions: vec![vec3(1.0, -1.0, 0.0), vec3(-1.0, 1.0, 0.0), vec3(-1.0, -1.0, 0.0), vec3(1.0, 1.0, 0.0)],
                uvs: vec![vec2(255, 0), vec2(0, 255), vec2(0, 0), vec2(255, 255)],
                ..Default::default()
            },
            vec![0, 1, 2, 0, 3, 1],
        );
        let quad = pipeline.meshes.insert(quad);
        /* #endregion */
        /* #region Lighting Shader */
        // Load the lighting pass shader
        let settings = ShaderInitSettings::default()
            .source("defaults/shaders/rendering/passthrough.vrsh.glsl")
            .source("defaults/shaders/rendering/lighting_pass.frsh.glsl")
            .directive("shadow_bias", Directive::Const(pipeline.settings().shadow_bias.to_string())); // TODO: FIX THIS
        let shader = pipeline.shaders.insert(Shader::new(settings).unwrap());
        /* #endregion */
        /* #region Deferred renderer init */
        let dimensions = TextureDimensions::Texture2d(pipeline.window.dimensions);

        // Since we use deferred rendering, we must create a new framebuffer for this renderer
        let mut framebuffer = 0;
        gl::GenFramebuffers(1, &mut framebuffer);
        gl::BindFramebuffer(gl::FRAMEBUFFER, framebuffer);
        // Create the textures now
        let texture_formats = [
            TextureFormat::RGB8R,
            TextureFormat::RGB32F,
            TextureFormat::RGB8RS,
            TextureFormat::RGB32F,
            TextureFormat::DepthComponent32,
        ];
        let texture_types = [DataType::U8, DataType::U8, DataType::U8, DataType::U8, DataType::F32];
        // Create all the textures at once
        let textures = texture_formats
            .into_iter()
            .zip(texture_types.into_iter())
            .map(|(_format, _type)| {
                pipeline
                    .textures
                    .insert(TextureBuilder::default().dimensions(dimensions)._format(_format)._type(_type).build())
            })
            .collect::<Vec<Handle<Texture>>>();

        // Now bind the texture attachememnts
        let attachements = [
            gl::COLOR_ATTACHMENT0,
            gl::COLOR_ATTACHMENT1,
            gl::COLOR_ATTACHMENT2,
            gl::COLOR_ATTACHMENT3,
            gl::DEPTH_ATTACHMENT,
        ];
        for (handle, &attachement) in textures.iter().zip(attachements.iter()) {
            let texture = pipeline.textures.get(handle).unwrap();
            gl::BindTexture(texture.target(), texture.oid());
            dbg!(attachement);
            dbg!(texture.target());
            dbg!(texture.oid());
            gl::FramebufferTexture2D(gl::FRAMEBUFFER, attachement, texture.target(), texture.oid(), 0);
        }

        // Note: the number of attachements are n-1 because we do not give it the gl::DEPTH_ATTACHEMENT
        gl::DrawBuffers(attachements.len() as i32 - 1, attachements.as_ptr() as *const u32);

        // Check frame buffer state
        if gl::CheckFramebufferStatus(gl::FRAMEBUFFER) != gl::FRAMEBUFFER_COMPLETE {
            panic!("Framebuffer has failed initialization! Error: '{:#x}'", gl::CheckFramebufferStatus(gl::FRAMEBUFFER));
        }

        // Unbind
        gl::BindFramebuffer(gl::FRAMEBUFFER, 0);
        /* #endregion */
        /* #region Others */
        let shadow_mapping = pipeline.settings().shadow_resolution.map(|resolution| ShadowMapping::new(pipeline, resolution));

        // Load the default sky gradient texture
        let sky_gradient = TextureBuilder::new(assetc::load::<Texture>("defaults/textures/sky_gradient.png").unwrap())
            .wrap_mode(TextureWrapping::ClampToBorder(None))
            .build();
        let sky_gradient = pipeline.textures.insert(sky_gradient);
        /* #endregion */
        println!("Successfully initialized the RenderPipeline Renderer!");
        Self {
            framebuffer,
            textures: textures.try_into().expect("Deferred textures count mismatch!"),
            lighting_pass: shader,
            quad,
            sky_gradient,
            shadow_mapping,
        }
    }
    // Resize the renderer's textures
    pub(crate) fn resize(&mut self, dimensions: veclib::Vector2<u16>, pipeline: &mut Pipeline) {
        // Very simple since we use an array
        for handle in self.textures.iter() {
            let mut texture = pipeline.textures.get_mut(handle).unwrap();
            texture.set_dimensions(TextureDimensions::Texture2d(dimensions)).unwrap();
        }
    }

    // Init OpenGL
    pub(crate) unsafe fn init_opengl() {
        gl::ClearColor(0.0, 0.0, 0.0, 1.0);
        gl::Enable(gl::DEPTH_TEST);
        gl::Enable(gl::CULL_FACE);
        gl::CullFace(gl::BACK);
    }

    // Prepare the FBO and clear the buffers
    pub(crate) unsafe fn prepare_for_rendering(&self, pipeline: &Pipeline) {
        gl::Viewport(0, 0, pipeline.window.dimensions.x as i32, pipeline.window.dimensions.y as i32);
        gl::BindFramebuffer(gl::FRAMEBUFFER, self.framebuffer);
        gl::Clear(gl::COLOR_BUFFER_BIT | gl::DEPTH_BUFFER_BIT);
    }

    // Render the whole scene
    pub fn render(&mut self, pipeline: &Pipeline, settings: RenderingSettings) {
        // Render normally
        for renderer in settings.normal {
            common::render_model(&settings, renderer, pipeline)
        }

        // Then render the shadows
        self.shadow_mapping.as_mut().map(|mapping| {
            // The first directional light that we find will be used as the sunlight
            let first = pipeline.lights
                .iter()
                .find(|(_, light)| 
                    light._type
                    .as_directional()
                    .is_some())
                .map(|(_, value)| value);
            
            let quat = first.map(|light| light.transform.rotation);
            quat.map(|quat| mapping.render_all_shadows(settings.shadowed, &quat, pipeline));
        });
    }

    /*
    // Get the fallback, default texture IDs in case the provided ones are not valid
    fn get_diffuse_map(pipeline: &Pipeline, material: &Material) -> ObjectID<Texture> {
        material.diffuse_map.get().map_or_else(|| pipeline.defaults.as_ref().unwrap().white, ObjectID::new)
    }
    fn get_normal_map(pipeline: &Pipeline, material: &Material) -> ObjectID<Texture> {
        material.normal_map.get().map_or_else(|| pipeline.defaults.as_ref().unwrap().normals_tex, ObjectID::new)
    }
    fn get_emissive_map(pipeline: &Pipeline, material: &Material) -> ObjectID<Texture> {
        material.emissive_map.get().map_or_else(|| pipeline.defaults.as_ref().unwrap().black, ObjectID::new)
    }
    // Setup uniforms for a specific renderer
    fn configure_uniforms<'a>(&self, pipeline: &'a Pipeline, renderer: &Renderer, pj_matrix: &veclib::Matrix4x4<f32>) -> Result<&'a Mesh, RenderingError> {
        // Pipeline data
        let material = pipeline.materials.get(renderer.material);
        // Load the default material if we don't have a valid one
        let material = material
            .or_else(|| {
                let id = pipeline.defaults.as_ref().unwrap().material;
                Some(pipeline.materials.get(id).unwrap())
            })
            .unwrap();

        // The shader will always be valid
        let shader = pipeline.shaders.get(material.shader).ok_or(RenderingError)?;
        let mesh = pipeline.meshes.get(renderer.mesh).ok_or(RenderingError)?;
        let mesh_matrix = &renderer.matrix;
        let settings = ShaderUniformsSettings::new(ShaderIDType::OpenGLID(shader.program));
        let uniforms = Uniforms::new(&settings, pipeline);
        // Bind first
        uniforms.bind_shader();
        // Then set the uniforms
        uniforms.set_mat44f32("project_view_matrix", *pj_matrix);
        uniforms.set_mat44f32("mesh_matrix", *mesh_matrix);
        // Optional
        material.uniforms.execute(&uniforms);
        renderer.uniforms.execute(&uniforms);
        // Textures might be not valid, so we fallback to the default ones just in case
        uniforms.set_texture("diffuse_tex", Self::get_diffuse_map(pipeline, material), 0);
        uniforms.set_texture("normals_tex", Self::get_normal_map(pipeline, material), 1);
        uniforms.set_texture("emissive_tex", Self::get_emissive_map(pipeline, material), 2);
        uniforms.set_vec3f32("tint", material.tint);
        uniforms.set_f32("normals_strength", material.normal_map_strength);
        uniforms.set_f32("emissive_strength", material.emissive_map_strength);
        uniforms.set_vec2f32("uv_scale", material.uv_scale);

        Ok(mesh)
    }
    // Render a single renderer
    fn render(mesh: &Mesh) {}

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
        let camera = &pipeline.camera;
        let pj_matrix = camera.projm * camera.viewm;
        for (_, renderer) in pipeline.renderers.iter() {
            // Check if we are visible
            if !renderer.flags.contains(RendererFlags::VISIBLE) {
                continue;
            }
            let result = self.configure_uniforms(pipeline, renderer, &pj_matrix);
            // The renderer might've failed setting it's uniforms
            if let Ok(mesh) = result {
                Self::render(mesh);
                debug_info.draw_calls += 1;
                debug_info.triangles += mesh.indices.len() as u64 / 3;
                debug_info.vertices += mesh.vertices.len() as u64 / 3;
            }
        }
    }
    // Render the scene's shadow maps
    fn render_scene_shadow_maps(shadow_mapping: &mut ShadowMapping, pipeline: &Pipeline, debug_info: &mut FrameDebugInfo) {
        // Change the cull face for better depth
        unsafe {
            gl::CullFace(gl::FRONT);
        }
        // Bind VBO and set light source
        shadow_mapping.bind_fbo(pipeline);
        let directional_light_source = pipeline.light_sources.get(pipeline.defaults.as_ref().unwrap().sun);
        if let Some(light) = directional_light_source {
            shadow_mapping.update_view_matrix(*light._type.as_directional().unwrap());
        }

        // Draw the renderers
        for (_, renderer) in pipeline.renderers.iter().filter(|(_, renderer)| renderer.flags.contains(RendererFlags::SHADOW_CASTER)) {
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
            gl::Viewport(0, 0, pipeline.window.dimensions.x as i32, pipeline.window.dimensions.y as i32);
        }
        // Get the pipeline data
        let camera = &pipeline.camera;

        // New uniforms
        let settings = ShaderUniformsSettings::new(ShaderIDType::ObjectID(self.lighting_pass));
        let uniforms = Uniforms::new(&settings, pipeline);
        self.bind_screen_quad_uniforms(uniforms, pipeline, camera);

        // Draw the quad
        let quad_mesh = pipeline.meshes.get(self.quad).unwrap();
        unsafe {
            // Draw to the postprocessing's framebuffer instead
            gl::BindFramebuffer(gl::FRAMEBUFFER, self.postprocessing.framebuffer);
            gl::Disable(gl::DEPTH_TEST);
            Self::render(quad_mesh);
        }
    }
    // Draw the postprocessing quad and render the color texture
    fn draw_postprocessing_quad(&mut self, pipeline: &Pipeline) {
        self.postprocessing.bind_fbo(pipeline, self);

        // Draw the quad
        let quad_mesh = pipeline.meshes.get(self.quad).unwrap();
        unsafe {
            // Draw to the postprocessing's framebuffer instead
            gl::BindFramebuffer(gl::FRAMEBUFFER, 0);
            Self::render(quad_mesh);
            gl::BindVertexArray(0);
            gl::Enable(gl::DEPTH_TEST);
        }
    }
    // Name is pretty self explanatory
    fn bind_screen_quad_uniforms(&mut self, uniforms: Uniforms, pipeline: &Pipeline, camera: &Camera) {
        uniforms.bind_shader();
        // The first directional light source is always the sun's light source
        let default = LightSource::new(LightSourceType::Directional {
            quat: veclib::Quaternion::<f32>::IDENTITY,
        });
        let light = pipeline.light_sources.get(pipeline.defaults.as_ref().unwrap().sun).unwrap_or(&default);
        let directional = light._type.as_directional().unwrap();
        uniforms.set_vec3f32("sunlight_dir", directional.mul_point(veclib::Vector3::Z));
        uniforms.set_f32("sunlight_strength", light.strength);
        uniforms.set_mat44f32(
            "lightspace_matrix",
            self.shadow_mapping
                .as_ref()
                .map_or(veclib::Matrix4x4::default(), |shadow_mapping| shadow_mapping.lightspace_matrix),
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
        let shadow_mapping_texture = self
            .shadow_mapping
            .as_ref()
            .map_or(pipeline.defaults.as_ref().unwrap().white, |shadow_mapping| shadow_mapping.depth_texture);
        uniforms.set_texture("shadow_map", shadow_mapping_texture, 6);
        uniforms.set_bool("shadows_enabled", self.shadow_mapping.is_some());
    }
    */
}
