use getset::Getters;
// Struct containing everything related to shadow mapping
// https://learnopengl.com/Advanced-Lighting/Shadows/Shadow-Mapping
use gl::types::GLuint;

use crate::{
    basics::{
        shader::{Shader, ShaderInitSettings},
        texture::{Texture, Texture2D, TextureFilter, TextureFlags, TextureFormat, TextureLayout, TextureParams, TextureWrapMode},
        uniforms::Uniforms,
    },
    pipeline::{Handle, Pipeline, ShadowSettings, FramebufferClearBits},
    utils::DataType,
};
use super::{ShadowedModel, Framebuffer};


// Shadow mapping for the main light source; the sun
#[derive(Getters)]
pub struct ShadowMapping {
    // Shadow map's main frame buffer
    framebuffer: Framebuffer,

    // Accumulated depth texture
    #[getset(get = "pub")]
    texture: Handle<Texture2D>,

    // Shader that we will use to render each object
    shader: Handle<Shader>,

    // Matrices
    ortho: vek::Mat4<f32>,
    #[getset(get = "pub")]
    lightspace: vek::Mat4<f32>,

    // Settings
    settings: ShadowSettings
}
impl ShadowMapping {
    // Initialize a new shadow mapper
    pub(crate) fn new(pipeline: &mut Pipeline, settings: ShadowSettings) -> Self {
        // Create the framebuffer
        let framebuffer = Framebuffer::new(pipeline, FramebufferClearBits::DEPTH);

        // Custom parameters for the shadow map texture
        let params = TextureParams {
            layout: TextureLayout {
                data: DataType::U8,
                internal_format: TextureFormat::DepthComponent16,
            },
            flags: TextureFlags::empty(),
            filter: TextureFilter::Nearest,
            wrap: TextureWrapMode::ClampToBorder(Some(vek::Rgba::<f32>::one())),
        };

        // Create the texture itself
        let texture = pipeline.insert(Texture2D::new(
            vek::Extent2::broadcast(settings.resolution.get().max(1)),
            None,
            params,
        ));

        // Now attach the depth texture (also set the draw and read buffers manually)
        framebuffer.bind_textures(pipeline, &[(texture, gl::DEPTH_ATTACHMENT)]);
        framebuffer.disable_draw_read_buffers();

        // Create the orthographic matrix
        // TODO: Cascaded shadow mapping
        const DIMS: f32 = 200.0;
        const NEAR: f32 = -2000.0;
        const FAR: f32 = 2000.0;
        let frustum = vek::FrustumPlanes {
            left: -DIMS,
            right: DIMS,
            bottom: -DIMS,
            top: DIMS,
            near: NEAR,
            far: FAR,
        };
        let ortho_matrix = vek::Mat4::<f32>::orthographic_rh_no(frustum);

        // Load our custom shadow shader
        let shader = pipeline.insert(Shader::new(
            ShaderInitSettings::default()
                .source("defaults/shaders/rendering/project.vrsh.glsl")
                .source("defaults/shaders/rendering/empty.frsh.glsl"),
        ).unwrap());

        Self {
            framebuffer,
            texture,
            ortho: ortho_matrix,
            shader,
            settings,
            lightspace: vek::Mat4::identity(),
        }
    }
    // Update the lightspace matrix
    pub(crate) fn update_matrix(&mut self, light_quat: vek::Quaternion<f32>) {
        // Update the light view matrix
        let matrix = vek::Mat4::from(light_quat);
        let forward = matrix.mul_direction(-vek::Vec3::unit_z());
        let up = matrix.mul_direction(vek::Vec3::unit_y());
        self.lightspace = self.ortho * vek::Mat4::look_at_rh(vek::Vec3::zero(), forward, up);
    }
    // Render the scene from the POV of the light source, so we can cast shadows
    pub(crate) unsafe fn render_all_shadows(&self, models: &[ShadowedModel], pipeline: &Pipeline) {
        // Draw into the shadow framebuffer
        gl::Viewport(0, 0, self.settings.resolution.get() as i32, self.settings.resolution.get() as i32);


        self.framebuffer.bind(|_| {
        
        });

        gl::BindFramebuffer(gl::FRAMEBUFFER, self.framebuffer);
        gl::Clear(gl::DEPTH_BUFFER_BIT);
        gl::Disable(gl::CULL_FACE);

        // Load the shader and it's uniforms
        let shader = pipeline.get(&self.shader).unwrap();
        Uniforms::new(shader.program(), pipeline, |mut uniforms| {
            // Render all the models
            for model in models {
                let (mesh, matrix) = (model.mesh, model.matrix);
                let mesh = pipeline.get(mesh).unwrap();

                // Calculate the light space matrix
                let lsm = self.lightspace * *matrix;

                // Pass the light space matrix to the shader
                uniforms.set_mat44f32("matrix", &lsm);

                // Render now
                super::common::render(mesh);
            }
        });

        // Reset the viewport to it's old values
        gl::BindFramebuffer(gl::FRAMEBUFFER, 0);
        gl::Viewport(0, 0, pipeline.window().dimensions().w as i32, pipeline.window().dimensions().h as i32);
        gl::Enable(gl::CULL_FACE);
    }
}
