// Struct containing everything related to shadow mapping
// https://learnopengl.com/Advanced-Lighting/Shadows/Shadow-Mapping
use gl::types::GLuint;

use crate::{
    basics::{
        mesh::Mesh,
        shader::{Shader, ShaderInitSettings},
        texture::{Texture, TextureBuilder, TextureDimensions, TextureFilter, TextureFormat, TextureWrapping},
        uniforms::Uniforms,
    },
    pipeline::{Handle, Pipeline},
};

use super::{RenderingError, ShadowedModel};
#[derive(Default)]
pub struct ShadowMapping {
    // Shadow-casting
    framebuffer: GLuint,
    depth_texture: Handle<Texture>,
    shader: Handle<Shader>,
    // Settings
    shadow_resolution: u16,
    // Light
    ortho_matrix: veclib::Matrix4x4<f32>,
    lightspace_matrix: veclib::Matrix4x4<f32>,
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
            .dimensions(TextureDimensions::Texture2d(veclib::vec2(shadow_resolution.max(1), shadow_resolution.max(1))))
            .filter(TextureFilter::Linear)
            .wrap_mode(TextureWrapping::ClampToBorder(Some(veclib::Vector4::<f32>::ONE)))
            .custom_params(&[(gl::TEXTURE_COMPARE_MODE, gl::COMPARE_REF_TO_TEXTURE), (gl::TEXTURE_COMPARE_FUNC, gl::GREATER)])
            ._format(TextureFormat::DepthComponent16)
            .build();
        let texture = pipeline.textures.insert(texture);
        // Now attach the depth texture
        unsafe {
            gl::BindFramebuffer(gl::FRAMEBUFFER, fbo);
            gl::FramebufferTexture2D(gl::FRAMEBUFFER, gl::DEPTH_ATTACHMENT, gl::TEXTURE_2D, pipeline.textures.get(&texture).unwrap().oid(), 0);
            gl::DrawBuffer(gl::NONE);
            gl::ReadBuffer(gl::NONE);
            // Unbind
            gl::BindFramebuffer(gl::FRAMEBUFFER, 0);
        }

        // Create the orthographic matrix
        const DIMS: f32 = 800.0;
        const NEAR: f32 = -2000.0;
        const FAR: f32 = 2000.0;
        let ortho_matrix = veclib::Matrix4x4::<f32>::from_orthographic(-DIMS, DIMS, -DIMS, DIMS, FAR, NEAR);

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
            ortho_matrix,
            shader,
            shadow_resolution,
            lightspace_matrix: veclib::Matrix4x4::IDENTITY,
        }
    }
    // Render the scene from the POV of the light source, so we can cast shadows
    pub(crate) fn render_all_shadows(
        &mut self,
        models: &[ShadowedModel],
        light_quat: &veclib::Quaternion<f32>,
        pipeline: &Pipeline,
    ) -> Result<(), RenderingError> {
        unsafe {
            // Setup the shadow framebuffer
            gl::Viewport(0, 0, self.shadow_resolution as i32, self.shadow_resolution as i32);
            gl::BindFramebuffer(gl::FRAMEBUFFER, self.framebuffer);
            gl::Clear(gl::DEPTH_BUFFER_BIT);
        }

        // Load the shader and it's uniforms
        let shader = pipeline.shaders.get(&self.shader).unwrap();
        let mut uniforms = Uniforms::new(shader.program(), pipeline, true);

        // Update the light view matrix
        let forward = light_quat.mul_point(veclib::Vector3::Z);
        let up = light_quat.mul_point(veclib::Vector3::Y);
        let view_matrix = veclib::Matrix4x4::<f32>::look_at(&forward, &up, &veclib::Vector3::ZERO);
        self.lightspace_matrix = self.ortho_matrix * view_matrix;

        // Render all the models
        for model in models {
            let (mesh, matrix) = (model.mesh, model.matrix);
            let mesh = pipeline.meshes.get(mesh).ok_or(RenderingError)?;

            // Calculate the light space matrix
            let lsm: veclib::Matrix4x4<f32> = self.lightspace_matrix * *matrix;

            // Pass the light space matrix to the shader
            uniforms.set_mat44f32("lsm_matrix", &lsm);

            // Render now
            unsafe { super::common::render(&mesh); }
        }
        Ok(())
    }
}
