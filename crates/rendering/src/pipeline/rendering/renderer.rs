use std::ffi::c_void;

use super::{common, RenderingSettings, ShadowMapping, cull_frustum, SceneRenderStats};
use crate::{
    basics::{
        mesh::{Mesh, Vertices},
        shader::{Directive, Shader, ShaderInitSettings},
        texture::{ResizableTexture, Texture2D, TextureFilter, TextureFlags, TextureFormat, TextureLayout, TextureParams, TextureWrapMode, CubeMap},
        uniforms::Uniforms,
    },
    pipeline::{Framebuffer, FramebufferClearBits, Handle, Pipeline},
    utils::{DataType, DEFAULT_WINDOW_SIZE},
};
use getset::{Getters, MutGetters};

// Scene renderer that will render our world using deferred rendering
// TODO: Document
#[derive(Getters, MutGetters)]
#[getset(get = "pub")]
pub struct SceneRenderer {
    // Default frame buffer
    #[getset(get_mut = "pub")]
    default: Framebuffer,

    // Deferred frame buffer
    #[getset(get_mut = "pub")]
    framebuffer: Framebuffer,

    // G-buffer textures
    textures: [Handle<Texture2D>; 6],

    // Screen rendering
    lighting: Handle<Shader>,
    quad: Handle<Mesh>,

    // Others
    skybox: Handle<CubeMap>,
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

        // Get the lighting shader's directives
        let s_settings = pipeline.settings().shadow();
        let s_bias = s_settings.map(|s| s.bias()).unwrap_or_default();
        let s_normal_offset = s_settings.map(|s| s.normal_offset()).unwrap_or_default();
        let s_samples = s_settings.map(|s| s.samples()).unwrap_or_default();

        // Load the lighting pass shader
        let settings = ShaderInitSettings::default()
            .source("defaults/shaders/rendering/uv_passthrough.vrsh.glsl")
            .source("defaults/shaders/rendering/lighting.frsh.glsl")
            .constant("shadow_bias", s_bias)
            .constant("normal_offset", s_normal_offset)
            .constant("samples", s_samples);
        let shader = pipeline.insert(Shader::new(settings).unwrap());
        /* #endregion */
        /* #region Deferred renderer init */
        let dimensions = pipeline.window().dimensions();

        // Since we use deferred rendering, we must create a new framebuffer for this renderer
        let mut framebuffer = Framebuffer::new(pipeline);

        // Create the textures now
        let texture_formats = [
            TextureFormat::RGB8R,
            TextureFormat::RGB32F,
            TextureFormat::RGB8RS,
            TextureFormat::RGB32F,
            TextureFormat::RGB8R,
            TextureFormat::DepthComponent32,
        ];
        let texture_types = [DataType::U8, DataType::U8, DataType::U8, DataType::U8, DataType::U8, DataType::F32];
        // Create all the textures at once
        let textures = texture_formats
            .into_iter()
            .zip(texture_types.into_iter())
            .map(|(internal_format, data_type)| {
                // Create a texture layout
                let layout = TextureLayout::new(data_type, internal_format);

                let params = TextureParams {
                    layout,
                    filter: TextureFilter::Nearest,
                    wrap: TextureWrapMode::Repeat,
                    flags: TextureFlags::RESIZABLE,
                };
                pipeline.insert(Texture2D::new(dimensions, None, params))
            })
            .collect::<Vec<Handle<Texture2D>>>();

        // Now bind the texture attachememnts
        let attachements = [
            gl::COLOR_ATTACHMENT0,
            gl::COLOR_ATTACHMENT1,
            gl::COLOR_ATTACHMENT2,
            gl::COLOR_ATTACHMENT3,
            gl::COLOR_ATTACHMENT4,
            gl::DEPTH_ATTACHMENT,
        ];

        // Bind textures
        let textures_and_attachements = textures.iter().cloned().zip(attachements).collect::<Vec<_>>();
        framebuffer.bind(|mut f| f.bind_textures(pipeline, &textures_and_attachements).unwrap());

        // Unbind
        gl::BindFramebuffer(gl::FRAMEBUFFER, 0);
        /* #endregion */
        /* #region Others */
        let shadow_mapping = pipeline.settings().shadow().map(|settings| ShadowMapping::new(pipeline, settings));
        
        // Load the default skybox by creating it from a HDR
        let hdr = pipeline.insert(assets::load_with::<Texture2D>("defaults/hdr/frozen_lake_4k.hdr", TextureParams::HDR_MAP_LOAD).unwrap());
        let skybox = pipeline.insert(CubeMap::from_equirectangular(hdr, 512));

        /* #endregion */
        println!("Successfully initialized the RenderPipeline Renderer!");
        Self {
            default: Framebuffer::from_raw_parts(pipeline, 0, DEFAULT_WINDOW_SIZE),
            framebuffer,
            textures: textures.try_into().expect("Deferred textures count mismatch!"),
            lighting: shader,
            quad,
            skybox,
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
    pub(crate) fn start_frame(&mut self, pipeline: &mut Pipeline) {
        self.framebuffer.bind(|mut f| {
            f.viewport(pipeline.window().dimensions());
            f.clear(FramebufferClearBits::COLOR | FramebufferClearBits::DEPTH);
        });
    }

    // Render the whole scene
    pub fn render(&mut self, pipeline: &Pipeline, mut settings: RenderingSettings) {
        // Scene statistics for the debugger
        let mut stats = SceneRenderStats { drawn: 0, culled: 0, shadowed: 0 };

        // Bind the deferred renderer's framebuffer
        self.framebuffer.bind(|_| {
            // AABB frustum culling cause I'm cool
            let taken = std::mem::take(&mut settings.normal);
            let objects = cull_frustum(pipeline.camera(), taken, &mut stats);

            // Render each object that isn't culled
            for renderer in objects {
                common::render_model(renderer, pipeline)
            }
        });

        // Then render the shadows
        if let Some(mapping) = &mut self.shadow_mapping {
            unsafe {
                // Update the lightspace matrix
                // The first directional light that we find will be used as the sunlight
                let first = settings.lights.iter().find_map(|(_type, params)| _type.as_directional().map(|_type| (_type, params)));

                if let Some((_parameters, transform)) = first {
                    // No need to update if nothing has changed
                    if settings.redraw_shadows {
                        // Only render directional shadow map if we have a sun
                        mapping.update_matrix(*transform.rotation);
                        // Then render shadows
                        mapping.render_all_shadows(&settings.shadowed, pipeline, &mut stats);
                    }
                }
            }
        }

        // Render the deferred quad
        unsafe {
            self.draw_deferred_quad(pipeline, settings);
        }

        // Store the stats in the pipeline
        *pipeline.stats().borrow_mut() = stats;
    }

    // Draw the deferred quad and do all lighting calculations inside it's fragment shader
    unsafe fn draw_deferred_quad(&mut self, pipeline: &Pipeline, settings: RenderingSettings) {
        // We have a ton of uniforms to set
        Uniforms::new(pipeline.get(&self.lighting).unwrap().program(), pipeline, |mut uniforms| {
            // Try to get the sunlight direction
            let first = settings.lights.iter().find_map(|(_type, params)| _type.as_directional().map(|_type| (_type, params)));
            let sunlight = first.map(|(params, transform)| (vek::Mat4::from(*transform.rotation).mul_direction(-vek::Vec3::unit_z()), params.strength));

            // Default sunlight values
            let sunlight = sunlight.unwrap_or((-vek::Vec3::unit_y(), 1.0));

            // Sunlight values
            uniforms.set_vec3f32("sunlight_dir", sunlight.0);
            uniforms.set_f32("sunlight_strength", sunlight.1);

            // Le day night system
            let time_of_day: f32 = sunlight.0.dot(-vek::Vec3::unit_y()) * 0.5 + 0.5;
            uniforms.set_f32("time_of_day", time_of_day);

            // Sunlight shadow mapping
            let default = vek::Mat4::<f32>::identity();
            let matrix = self.shadow_mapping.as_ref().map(|mapping| mapping.lightspace()).unwrap_or(&default);
            uniforms.set_mat44f32("lightspace_matrix", matrix);

            // Set the camera matrices and camera values
            let inverse_pr_m = (vek::Mat4::<f32>::from(pipeline.camera().rotation)) * pipeline.camera().proj.inverted();
            uniforms.set_mat44f32("inverse_pr_matrix", &inverse_pr_m);
            uniforms.set_mat44f32("pv_matrix", &pipeline.camera().proj_view);
            uniforms.set_vec3f32("camera_pos", pipeline.camera().position);
            uniforms.set_vec3f32("camera_dir", pipeline.camera().forward);

            // Also gotta set the deferred textures
            // &str array because I am lazy
            let names = ["diffuse_texture", "emissive_texture", "normals_texture", "position_texture", "mask_texture", "depth_texture"];
            // Set each texture
            for (name, handle) in names.into_iter().zip(self.textures.iter()) {
                uniforms.set_texture2d(name, handle);
            }

            // Skybox cubemap
            uniforms.set_cubemap("skybox", &self.skybox);

            // If we have shadow mapping disabled we must use the default white texture
            let shadow_mapping_texture = self
                .shadow_mapping
                .as_ref()
                .map_or(&pipeline.defaults().white, |shadow_mapping| &shadow_mapping.texture());
            uniforms.set_texture2d("shadow_map", shadow_mapping_texture);
            uniforms.set_bool("shadows_enabled", self.shadow_mapping.is_some());
        });

        // Draw the quad
        let quad_mesh = pipeline.get(&self.quad).unwrap();
        // Draw to the default framebuffer, and keep it bound
        self.default.bind(|mut bound| {
            gl::Disable(gl::DEPTH_TEST);
            common::render(quad_mesh);
            bound.viewport(pipeline.window().dimensions());
            gl::Enable(gl::DEPTH_TEST);
            gl::BindVertexArray(0);
        });
    }

    // Screenshot the current frame
    // This must be done after we render everything
    pub fn screenshot(&mut self, dimensions: vek::Extent2<u32>) -> Vec<u8> {
        // Create a vector that'll hod all of our RGB bytes
        let bytes_num = dimensions.as_::<usize>().product() * 3;
        let mut bytes = vec![0; bytes_num];
        // Read
        unsafe {
            gl::ReadPixels(
                0,
                0,
                dimensions.w as i32,
                dimensions.h as i32,
                gl::RGB,
                gl::UNSIGNED_BYTE,
                bytes.as_mut_ptr() as *mut c_void,
            );
            gl::Finish();
        }
        bytes
    }
}
