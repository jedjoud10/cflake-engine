use crate::RenderPipeline;
// Static mut RenderPipeline
pub static mut render_pipeline: RenderPipeline = RenderPipeline::default();

pub mod pipec {
    use crate::{render_pipeline, GPUObject, RenderTask, Shader, SharedData, SubShader, Texture};
    // Start the render pipeline by initializing OpenGL on the new render thread
    pub fn init_pipeline(glfw: &mut glfw::Glfw, window: &mut glfw::Window) {
        unsafe {
            render_pipeline.init_pipeline(glfw, window);
        }
    }

    // Actual commands start here
    pub fn create_texture(texture: Texture) -> GPUObject {
        unsafe { render_pipeline.task_immediate(RenderTask::TextureCreate(SharedData::new(texture))) }
    }
    pub fn create_subshader(subshader: SubShader) -> GPUObject {
        unsafe { render_pipeline.task_immediate(RenderTask::SubShaderCreate(SharedData::new(subshader))) }
    }
    pub fn create_shader(shader: Shader) -> GPUObject {
        unsafe { render_pipeline.task_immediate(RenderTask::ShaderCreate(SharedData::new(shader))) }
    }
}
