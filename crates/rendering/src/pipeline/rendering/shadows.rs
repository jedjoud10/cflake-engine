// Struct containing everything related to shadow mapping
// https://learnopengl.com/Advanced-Lighting/Shadows/Shadow-Mapping
use gl::types::GLuint;

use crate::{
    basics::{
        shader::{Shader, ShaderInitSettings},
        texture::{Texture, TextureBuilder, TextureFilter, TextureFormat, TextureLayout, TextureWrapMode, TextureFlags, Texture2D, TextureParams},
        uniforms::Uniforms,
    },
    pipeline::{Handle, Pipeline},
    utils::DataType,
};

use super::{RenderingError, ShadowedModel};
#[derive(Default)]
pub struct ShadowMapping {
    // Shadow-casting
    framebuffer: GLuint,
    pub(crate) depth_texture: Handle<Texture2D>,
    shader: Handle<Shader>,

    // Matrices
    ortho: vek::Mat4<f32>,
    pub(crate) lightspace: vek::Mat4<f32>,

    // Settings
    shadow_resolution: u16,
}
impl ShadowMapping {
    // Initialize a new shadow mapper
    pub(crate) fn new(pipeline: &mut Pipeline, shadow_resolution: u16) -> Self {
        // Create the framebuffer
        let fbo = unsafe {
            let mut fbo = 0;
            gl::GenFramebuffers(1, &mut fbo);
            fbo
        };

        // Create the depth texture
        let texture = TextureBuilder::default()
            .dimensions(vek::Vec2::broadcast(shadow_resolution.max(1)))
            .params(TextureParams {
                layout: TextureLayout { data: DataType::U8, internal_format: TextureFormat::DepthComponent16 },
                filter: TextureFilter::Linear,
                wrap: TextureWrapMode::ClampToBorder(Some(vek::Vec4::<f32>::one())),
                ..Default::default()
            })
            .build();
        let texture = pipeline.textures.insert(texture);
        // Now attach the depth texture
        unsafe {
            gl::BindFramebuffer(gl::FRAMEBUFFER, fbo);
            gl::FramebufferTexture2D(gl::FRAMEBUFFER, gl::DEPTH_ATTACHMENT, gl::TEXTURE_2D, pipeline.textures.get(&texture).unwrap().texture(), 0);
            gl::DrawBuffer(gl::NONE);
            gl::ReadBuffer(gl::NONE);
            // Unbind
            gl::BindFramebuffer(gl::FRAMEBUFFER, 0);
        }

        // Create the orthographic matrix
        const DIMS: f32 = 800.0;
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
        let shader = Shader::new(
            ShaderInitSettings::default()
                .source("defaults/shaders/rendering/shadow.vrsh.glsl")
                .source("defaults/shaders/rendering/shadow.frsh.glsl"),
        )
        .unwrap();
        let shader = pipeline.shaders.insert(shader);

        Self {
            framebuffer: fbo,
            depth_texture: texture,
            ortho: ortho_matrix,
            shader,
            shadow_resolution,
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
    pub(crate) unsafe fn render_all_shadows(&self, models: &[ShadowedModel], pipeline: &Pipeline) -> Result<(), RenderingError> {
        // Setup the shadow framebuffer
        gl::Viewport(0, 0, self.shadow_resolution as i32, self.shadow_resolution as i32);
        gl::BindFramebuffer(gl::FRAMEBUFFER, self.framebuffer);
        gl::Clear(gl::DEPTH_BUFFER_BIT);
        gl::Disable(gl::CULL_FACE);

        // Load the shader and it's uniforms
        let shader = pipeline.shaders.get(&self.shader).unwrap();
        let mut uniforms = Uniforms::new(shader.program(), pipeline, true);

        // Render all the models
        for model in models {
            let (mesh, matrix) = (model.mesh, model.matrix);
            let mesh = pipeline.meshes.get(mesh).ok_or(RenderingError)?;

            // Calculate the light space matrix
            let lsm = self.lightspace * *matrix;

            // Pass the light space matrix to the shader
            uniforms.set_mat44f32("lsm_matrix", &lsm);

            // Render now
            super::common::render(mesh);
        }

        // Reset
        gl::BindFramebuffer(gl::FRAMEBUFFER, 0);
        gl::Viewport(0, 0, pipeline.window().dimensions().x as i32, pipeline.window().dimensions().y as i32);
        gl::Enable(gl::CULL_FACE);

        Ok(())
    }
}
