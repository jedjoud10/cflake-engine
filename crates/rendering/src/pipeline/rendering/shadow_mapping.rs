use crate::{
    basics::{
        mesh::Mesh,
        renderer::Renderer,
        shader::{Shader, ShaderSettings},
        texture::{Texture, TextureFilter, TextureFormat, TextureType, TextureWrapping},
        uniforms::{ShaderIDType, ShaderUniformsSettings, Uniforms},
    },
    object::ObjectID,
    pipeline::{pipec, InternalPipeline, Pipeline},
};

use super::error::RenderingError;
use super::scene_renderer::*;
// Struct containing everything related to shadow mapping
// https://learnopengl.com/Advanced-Lighting/Shadows/Shadow-Mapping
#[derive(Default)]
pub struct ShadowMapping {
    // Main
    pub framebuffer: u32,
    pub(crate) depth_texture: ObjectID<Texture>,
    pub ortho_matrix: veclib::Matrix4x4<f32>,
    pub shadow_shader: ObjectID<Shader>,
    pub(crate) lightspace_matrix: veclib::Matrix4x4<f32>,
    pub shadow_resolution: u16,
}
impl ShadowMapping {
    // Setup uniforms for a specific renderer when rendering shadows
    pub(crate) fn configure_uniforms<'a>(
        &self,
        pipeline: &'a Pipeline,
        renderer: &Renderer,
    ) -> Result<&'a Mesh, RenderingError> {
        // Always use our internal shadow shader
        let mesh = pipeline.meshes.get(renderer.mesh).ok_or(RenderingError)?;
        let model_matrix = &renderer.matrix;

        // Calculate the light space matrix
        let lsm: veclib::Matrix4x4<f32> = self.lightspace_matrix * *model_matrix;

        // Pass the light space matrix to the shader
        let settings = ShaderUniformsSettings::new(ShaderIDType::ObjectID(self.shadow_shader));
        let group = Uniforms::new(&settings, pipeline);
        // Update the uniforms
        group.set_mat44f32("lsm_matrix", lsm);

        Ok(mesh)
    }
    // Initialize a new shadow mapper
    pub(crate) fn new(
        renderer: &mut SceneRenderer,
        shadow_resolution: u16,
        internal: &mut InternalPipeline,
        pipeline: &mut Pipeline,
    ) -> Self {
        // Create the framebuffer
        let fbo = unsafe {
            let mut fbo = 0;
            gl::GenFramebuffers(1, &mut fbo);
            fbo
        };

        // Create the depth texture
        let texture = Texture::default()
            .with_dimensions(TextureType::Texture2D(
                shadow_resolution.max(1),
                shadow_resolution.max(1),
            ))
            .with_filter(TextureFilter::Linear)
            .with_wrapping_mode(TextureWrapping::ClampToBorder)
            .with_border_colors([veclib::Vector4::<f32>::ONE; 4])
            .with_custom_gl_param(gl::TEXTURE_COMPARE_MODE, gl::COMPARE_REF_TO_TEXTURE)
            .with_custom_gl_param(gl::TEXTURE_COMPARE_FUNC, gl::GREATER)
            .with_format(TextureFormat::DepthComponent16);
        let texture = pipec::construct(pipeline, texture).unwrap();
        pipeline.flush(internal, renderer);
        // Now attach the depth texture
        unsafe {
            gl::BindFramebuffer(gl::FRAMEBUFFER, fbo);
            gl::FramebufferTexture2D(
                gl::FRAMEBUFFER,
                gl::DEPTH_ATTACHMENT,
                gl::TEXTURE_2D,
                pipeline.textures.get(texture).unwrap().oid,
                0,
            );
            gl::DrawBuffer(gl::NONE);
            gl::ReadBuffer(gl::NONE);
            // Unbind
            gl::BindFramebuffer(gl::FRAMEBUFFER, 0);
        }
        // Create some matrices
        const DIMS: f32 = 800.0;
        const NEAR: f32 = -2000.0;
        const FAR: f32 = 2000.0;
        let ortho_matrix =
            veclib::Matrix4x4::<f32>::from_orthographic(-DIMS, DIMS, -DIMS, DIMS, FAR, NEAR);

        // Load our custom shadow shader
        let shader = Shader::new(
            ShaderSettings::default()
                .source("defaults/shaders/rendering/shadow.vrsh.glsl")
                .source("defaults/shaders/rendering/shadow.frsh.glsl"),
        )
        .unwrap();
        let shader = pipec::construct(pipeline, shader).unwrap();
        pipeline.flush(internal, renderer);
        Self {
            framebuffer: fbo,
            depth_texture: texture,
            ortho_matrix,
            shadow_shader: shader,
            shadow_resolution,
            lightspace_matrix: veclib::Matrix4x4::IDENTITY,
        }
    }
    // Update the internally stored view matrix with the new direction of our sun
    pub(crate) fn update_view_matrix(&mut self, new_quat: veclib::Quaternion<f32>) {
        let forward = new_quat.mul_point(veclib::Vector3::Z);
        let up = new_quat.mul_point(veclib::Vector3::Y);
        let view_matrix = veclib::Matrix4x4::<f32>::look_at(&forward, &up, &veclib::Vector3::ZERO);
        self.lightspace_matrix = self.ortho_matrix * view_matrix;
        /*
        const DIMS: f32 = 400.0;
        const NEAR: f32 = -2000.0;
        const FAR: f32 = 2000.0;
        self.ortho_matrix = veclib::Matrix4x4::<f32>::from_orthographic(-DIMS - camera.position.z, DIMS - camera.position.z, -DIMS + camera.position.x, DIMS + camera.position.x, FAR, NEAR);;
        */
    }
    // Make sure we are ready to draw shadows
    pub(crate) fn bind_fbo(&self, pipeline: &Pipeline) {
        unsafe {
            gl::Viewport(
                0,
                0,
                self.shadow_resolution as i32,
                self.shadow_resolution as i32,
            );
            gl::BindFramebuffer(gl::FRAMEBUFFER, self.framebuffer);
            gl::Clear(gl::DEPTH_BUFFER_BIT);
            let settings = ShaderUniformsSettings::new(ShaderIDType::ObjectID(self.shadow_shader));
            let group = Uniforms::new(&settings, pipeline);
            group.bind_shader();
        }
    }
}
