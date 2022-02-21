use gl::types::GLuint;

use crate::{
    basics::{
        shader::{Shader, ShaderSettings},
        texture::{Texture, TextureFilter, TextureFormat, TextureType},
        uniforms::{ShaderIDType, ShaderUniformsSettings, Uniforms},
    },
    object::ObjectID,
    pipeline::{pipec, InternalPipeline, Pipeline, PipelineRenderer},
};

// Post processing effects that are rendered to the final frame buffer
#[derive(Default)]
pub struct PostProcessing {
    pub(crate) framebuffer: GLuint,
    color_texture: ObjectID<Texture>,
    shader: ObjectID<Shader>,
}

impl PostProcessing {
    // Initialize a new post processing effects handler
    pub(crate) fn new(renderer: &mut PipelineRenderer, internal: &mut InternalPipeline, pipeline: &mut Pipeline, dims: TextureType) -> Self {
        // Create the framebuffer
        let fbo = unsafe {
            let mut fbo = 0;
            gl::GenFramebuffers(1, &mut fbo);
            fbo
        };
        // Create the final color texture
        let texture = Texture::default()
            .with_filter(TextureFilter::Linear)
            .with_dimensions(dims)
            .with_format(TextureFormat::RGB16F);
        let texture = pipec::construct(pipeline, texture).unwrap();

        // Flush
        pipeline.flush(internal, renderer);

        // Now attach the color texture
        unsafe {
            gl::BindFramebuffer(gl::FRAMEBUFFER, fbo);
            gl::FramebufferTexture2D(gl::FRAMEBUFFER, gl::COLOR_ATTACHMENT0, gl::TEXTURE_2D, pipeline.textures.get(texture).unwrap().oid, 0);
            let attachements = vec![gl::COLOR_ATTACHMENT0];
            gl::DrawBuffers(attachements.len() as i32, attachements.as_ptr() as *const u32);
            // Check frame buffer state
            if gl::CheckFramebufferStatus(gl::FRAMEBUFFER) != gl::FRAMEBUFFER_COMPLETE {
                panic!("Framebuffer has failed initialization! Error: '{:#x}'", gl::CheckFramebufferStatus(gl::FRAMEBUFFER));
            }

            // Unbind
            gl::BindFramebuffer(gl::FRAMEBUFFER, 0);
        }
        // Load the final post processing shader
        let shader = Shader::new(
            ShaderSettings::default()
                .source("defaults/shaders/rendering/passthrough.vrsh.glsl")
                .source("defaults/shaders/rendering/postprocessing_pass.frsh.glsl"),
        )
        .unwrap();
        let shader = pipec::construct(pipeline, shader).unwrap();
        pipeline.flush(internal, renderer);
        Self {
            framebuffer: fbo,
            color_texture: texture,
            shader,
        }
    }
    // Resize the color texture
    pub(crate) fn resize_texture(&mut self, dims: TextureType, pipeline: &mut Pipeline) {
        let color_texture = pipeline.textures.get_mut(self.color_texture).unwrap();
        color_texture.update_size_fill(dims, Vec::new()).unwrap();
    }
    // Make sure we are ready to draw post processing effects
    pub(crate) fn bind_fbo(&self, pipeline: &Pipeline, render: &PipelineRenderer) {
        unsafe {
            gl::BindFramebuffer(gl::FRAMEBUFFER, self.framebuffer);
            //gl::Clear(gl::COLOR | gl::DEPTH_BUFFER_BIT);
            let settings = ShaderUniformsSettings::new(ShaderIDType::ObjectID(self.shader));
            let uniforms = Uniforms::new(&settings, pipeline);
            uniforms.bind_shader();
            // Kill me
            uniforms.set_texture("color_texture", self.color_texture, 0);
            let camera = &pipeline.camera;
            uniforms.set_mat44f32("pv_matrix", camera.projm * camera.viewm);
            let pr_m = camera.projm * (veclib::Matrix4x4::<f32>::from_quaternion(&camera.rotation));
            uniforms.set_mat44f32("pr_matrix", pr_m);
            uniforms.set_vec2f32("nf_planes", camera.clip_planes);
            uniforms.set_texture("normals_texture", render.normals_texture, 1);
            uniforms.set_texture("position_texture", render.position_texture, 2);
            uniforms.set_texture("depth_texture", render.depth_texture, 3);
        }
    }
}
