use std::ffi::c_void;

use super::{common, RenderingSettings, ShadowMapping};
use crate::{
    basics::{
        mesh::{Mesh, Vertices},
        shader::{Directive, Shader, ShaderInitSettings},
        texture::{ResizableTexture, Texture, Texture2D, TextureBuilder, TextureFlags, TextureFormat, TextureLayout, TextureParams, TextureWrapMode},
        uniforms::Uniforms,
    },
    pipeline::{Handle, Pipeline},
    utils::{DataType},
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
    textures: [Handle<Texture2D>; 5],

    // Screen rendering
    lighting: Handle<Shader>,
    quad: Handle<Mesh>,

    // Others
    sky_gradient: Handle<Texture2D>,
    shadow_mapping: Option<ShadowMapping>,
}

impl SceneRenderer {
    // Initialize a new scene renderer
    pub(crate) unsafe fn new(pipeline: &mut Pipeline) -> Self {
        println!("Initializing the scene renderer...");
        /* #region Quad */
        // Create the quad mesh that we will use to render the whole screen
        let quad = Mesh::new(
            Vertices {
                positions: vec![
                    vek::Vec3::new(1.0, -1.0, 0.0),
                    vek::Vec3::new(-1.0, 1.0, 0.0),
                    vek::Vec3::new(-1.0, -1.0, 0.0),
                    vek::Vec3::new(1.0, 1.0, 0.0),
                ],
                uvs: vec![vek::Vec2::new(255, 0), vek::Vec2::new(0, 255), vek::Vec2::new(0, 0), vek::Vec2::new(255, 255)],
                ..Default::default()
            },
            vec![0, 1, 2, 0, 3, 1],
        );
        let quad = pipeline.insert(quad);
        /* #endregion */
        /* #region Lighting Shader */
        // Load the lighting pass shader
        let settings = ShaderInitSettings::default()
            .source("defaults/shaders/rendering/uv_passthrough.vrsh.glsl")
            .source("defaults/shaders/rendering/lighting_pass.frsh.glsl")
            .directive("shadow_bias", Directive::Const(pipeline.settings().shadow_bias.to_string())); // TODO: FIX THIS
        let shader = pipeline.insert(Shader::new(settings).unwrap());
        /* #endregion */
        /* #region Deferred renderer init */
        let dimensions = pipeline.window().dimensions();

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
            .map(|(internal_format, data_type)| {
                // Create a texture layout
                let layout = TextureLayout::new(data_type, internal_format);

                pipeline.insert(
                    TextureBuilder::default()
                        .dimensions(dimensions)
                        .params(TextureParams {
                            layout,
                            flags: TextureFlags::RESIZABLE,
                            ..Default::default()
                        })
                        .build(),
                )
            })
            .collect::<Vec<Handle<Texture2D>>>();

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
            gl::BindTexture(texture.target().unwrap(), texture.name().unwrap());
            gl::FramebufferTexture2D(gl::FRAMEBUFFER, attachement, texture.target().unwrap(), texture.name().unwrap(), 0);
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
        let sky_gradient = TextureBuilder::new(assetc::load::<Texture2D>("defaults/textures/sky_gradient.png").unwrap())
            .params(TextureParams {
                wrap: TextureWrapMode::ClampToEdge(None),
                ..Default::default()
            })
            .build();
        let sky_gradient = pipeline.insert(sky_gradient);
        /* #endregion */
        println!("Successfully initialized the RenderPipeline Renderer!");
        Self {
            framebuffer,
            textures: textures.try_into().expect("Deferred textures count mismatch!"),
            lighting: shader,
            quad,
            sky_gradient,
            shadow_mapping,
        }
    }
    // Resize the renderer's textures
    pub(crate) fn resize(&mut self, pipeline: &mut Pipeline) {
        // Very simple since we use an array
        let dimensions = pipeline.window().dimensions();
        for handle in self.textures.iter() {
            let texture = pipeline.get_mut(handle).unwrap();
            texture.resize(dimensions).unwrap();
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
    pub(crate) unsafe fn start_frame(&mut self, pipeline: &mut Pipeline) {
        gl::Viewport(0, 0, pipeline.window().dimensions().w as i32, pipeline.window().dimensions().h as i32);
        gl::BindFramebuffer(gl::FRAMEBUFFER, self.framebuffer);
        gl::Clear(gl::COLOR_BUFFER_BIT | gl::DEPTH_BUFFER_BIT);
    }

    // Render the whole scene
    pub fn render(&mut self, pipeline: &Pipeline, settings: RenderingSettings) {
        // Render normally
        let mut last_material = Handle::default();
        for renderer in settings.normal {
            common::render_model(&settings, renderer, &mut last_material, pipeline)
        }

        // Then render the shadows
        if let Some(mapping) = &mut self.shadow_mapping {
            unsafe {
                // Update the lightspace matrix
                // The first directional light that we find will be used as the sunlight
                let first = settings.lights.iter().find_map(|(_type, params)| _type.as_directional().map(|_type| (_type, params)));
    
                if let Some((_parameters, transform)) = first {
                    // Only render directional shadow map if we have a sun
                    mapping.update_matrix(*transform.rotation);
                    // Then render shadows
                    mapping.render_all_shadows(settings.shadowed, pipeline).unwrap();
                }
            }
        }

        // Render the deferred quad
        unsafe {
            self.draw_deferred_quad(pipeline, settings);
        }
    }

    // Draw the deferred quad and do all lighting calculations inside it's fragment shader
    unsafe fn draw_deferred_quad(&self, pipeline: &Pipeline, settings: RenderingSettings) {
        // We have a ton of uniforms to set
        let mut uniforms = Uniforms::new(pipeline.shaders.get(&self.lighting).unwrap().program(), pipeline);

        // Try to get the sunlight direction
        let first = settings.lights.iter().find_map(|(_type, params)| _type.as_directional().map(|_type| (_type, params)));
        let sunlight = first.map(|(params, transform)| (vek::Mat4::from(*transform.rotation).mul_direction(vek::Vec3::unit_z()), params.strength));

        // Default sunlight values
        let sunlight = sunlight.unwrap_or((vek::Vec3::unit_y(), 1.0));

        // Sunlight values
        uniforms.set_vec3f32("sunlight_dir", sunlight.0);
        uniforms.set_f32("sunlight_strength", sunlight.1);

        // Sunlight shadow mapping
        let default = vek::Mat4::<f32>::identity();
        let matrix = self.shadow_mapping.as_ref().map(|mapping| &mapping.lightspace).unwrap_or(&default);
        uniforms.set_mat44f32("lightspace_matrix", matrix);

        // Set the camera matrices
        let inverse_pr_m = (vek::Mat4::<f32>::from(pipeline.camera().rotation)) * pipeline.camera().projm.inverted();
        uniforms.set_mat44f32("inverse_pr_matrix", &inverse_pr_m);
        uniforms.set_mat44f32("pv_matrix", &pipeline.camera().projm_viewm);

        // Also gotta set the deferred textures
        // &str array because I am lazy
        let names = ["diffuse_texture", "emissive_texture", "normals_texture", "position_texture", "depth_texture"];
        // Set each texture
        for (name, handle) in names.into_iter().zip(self.textures.iter()) {
            uniforms.set_texture2d(name, handle);
        }

        // Sky gradient texture
        uniforms.set_texture2d("sky_gradient", &self.sky_gradient);

        // If we have shadow mapping disabled we must use the default white texture
        let shadow_mapping_texture = self
            .shadow_mapping
            .as_ref()
            .map_or(&pipeline.defaults().white, |shadow_mapping| &shadow_mapping.depth_texture);
        uniforms.set_texture2d("shadow_map", shadow_mapping_texture);
        uniforms.set_bool("shadows_enabled", self.shadow_mapping.is_some());

        // Draw the quad
        let quad_mesh = pipeline.meshes.get(&self.quad).unwrap();
        // Draw to the deferred framebuffer instead
        gl::BindFramebuffer(gl::FRAMEBUFFER, 0);
        gl::Disable(gl::DEPTH_TEST);
        common::render(quad_mesh);
        gl::Enable(gl::DEPTH_TEST);
        gl::BindVertexArray(0);
    }

    // Screenshot the current frame
    // This must be done after we render everything
    pub fn screenshot(&mut self, dimensions: vek::Extent2<u16>) -> Vec<u8> {
        // Create a vector that'll hod all of our RGB bytes
        let bytes_num = dimensions.as_::<usize>().product() * 3;
        let mut bytes = vec![0; bytes_num];
        // Read
        unsafe {
            gl::ReadPixels(0, 0, dimensions.w as i32, dimensions.h as i32, gl::RGB, gl::UNSIGNED_BYTE, bytes.as_mut_ptr() as *mut c_void);
            gl::Finish();
        }
        bytes
    }
}
