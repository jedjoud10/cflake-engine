use crate::{
    basics::{texture::{Texture, TextureFilter, TextureFormat, TextureType, TextureWrapping}, renderer::Renderer, model::ModelBuffers, material::Material, uniforms::{ShaderUniformsGroup, ShaderUniformsSettings, ShaderIdentifier}, shader::{Shader, ShaderSettings}},
    object::ObjectID,
    pipeline::{pipec, InternalPipeline, Pipeline},
};

use super::PipelineRenderer;

// Struct containing everything related to shadow mapping
// https://learnopengl.com/Advanced-Lighting/Shadows/Shadow-Mapping
#[derive(Default)]
pub struct ShadowMapping {
    // Main
    pub(crate) framebuffer: u32,
    pub(crate) depth_texture: ObjectID<Texture>,
    pub(crate) ortho_matrix: veclib::Matrix4x4<f32>,
    pub(crate) view_matrix: veclib::Matrix4x4<f32>,
    pub(crate) shadow_shader: ObjectID<Shader>,
}

const SHADOW_RES: u16 = 512;
impl ShadowMapping {
    // Setup uniforms for a specific renderer when rendering shadows
    pub(crate) fn configure_uniforms<'a>(&self, pipeline: &'a Pipeline, renderer: &Renderer) -> Option<(&'a ModelBuffers, usize)> {
        // Always use our internal shadow shader
        let shader = self.shadow_shader;
        let model = pipeline.get_model(renderer.model)?;
        let model_matrix = &renderer.matrix;

        // Calculate the light space matrix
        let lightspace_matrix: veclib::Matrix4x4<f32> = self.ortho_matrix * self.view_matrix * *model_matrix;

        // Pass the light space matrix to the shader
        let settings = ShaderUniformsSettings::new(ShaderIdentifier::ObjectID(shader));
        let mut group = ShaderUniformsGroup::new();
        group.set_mat44f32("lightspace_matrix", lightspace_matrix);

        // Update the uniforms
        group.execute(pipeline, settings).unwrap();
        Some((&model.1, model.0.triangles.len()))
    }
    // Initialize a new shadow mapper
    pub(crate) fn new(renderer: &mut PipelineRenderer, internal: &mut InternalPipeline, pipeline: &mut Pipeline) -> Self {
        // Create the framebuffer
        let fbo = unsafe {
            let mut fbo = 0;
            gl::GenFramebuffers(1, &mut fbo);
            fbo
        };
        // Create the depth texture
        let texture = Texture::default()
            .set_dimensions(TextureType::Texture2D(SHADOW_RES, SHADOW_RES))
            .set_filter(TextureFilter::Nearest)
            .set_wrapping_mode(TextureWrapping::Repeat)
            .set_format(TextureFormat::DepthComponent32);
        let texture = pipec::construct(pipeline, texture).unwrap();
        pipeline.flush(internal, renderer);
        // Now attach the depth texture
        unsafe {
            gl::BindFramebuffer(gl::FRAMEBUFFER, fbo);
            gl::FramebufferTexture2D(gl::FRAMEBUFFER, gl::DEPTH_ATTACHMENT, gl::TEXTURE_2D, pipeline.get_texture(texture).unwrap().oid, 0);
            gl::DrawBuffer(gl::NONE);
            gl::ReadBuffer(gl::NONE);
            // Unbind
            gl::BindFramebuffer(gl::FRAMEBUFFER, 0);
        }
        // Create some matrices
        const DIMS: f32 = 40.0;
        const NEAR: f32 = 1.0;
        const FAR: f32 = 160.0;
        let ortho_matrix = veclib::Matrix4x4::<f32>::from_orthographic(-DIMS, DIMS, -DIMS, DIMS, FAR, NEAR);

        // Load our custom shadow shader
        let shader = Shader::new(ShaderSettings::default()
            .source("defaults\\shaders\\rendering\\shadow.vrsh.glsl")
            .source("defaults\\shaders\\rendering\\shadow.frsh.glsl")
        ).unwrap();
        let shader = pipec::construct(pipeline, shader).unwrap();
        pipeline.flush(internal, renderer);
        Self {
            framebuffer: fbo,
            depth_texture: texture,
            ortho_matrix,
            view_matrix: veclib::Matrix4x4::IDENTITY,
            shadow_shader: shader, 
        }
    }
    // Update the internally stored view matrix with the new direction of our sun
    pub(crate) fn update_view_matrix(&mut self, new_quat: veclib::Quaternion<f32>) {
        let forward = new_quat.mul_point(veclib::Vector3::Z);
        let up = new_quat.mul_point(veclib::Vector3::Y);
        self.view_matrix = veclib::Matrix4x4::<f32>::look_at(&(forward * 20.0), &up, &veclib::Vector3::ZERO)
    }
    // Make sure we are ready to draw shadows
    pub(crate) fn bind_fbo(&self) {
        unsafe {
            gl::Viewport(0, 0, SHADOW_RES as i32, SHADOW_RES as i32);
            gl::BindFramebuffer(gl::FRAMEBUFFER, self.framebuffer);
            gl::Clear(gl::DEPTH_BUFFER_BIT);
        }
    }    
}
