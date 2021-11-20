use crate::RenderPipeline;
// Static mut RenderPipeline
pub static mut render_pipeline: RenderPipeline = RenderPipeline::default();

pub mod pipeline_commands {
    use crate::{GPUObject, RenderTask, SharedGPUObject, SubShader, Texture, render_pipeline};
    // Start the render pipeline by initializing OpenGL on the new render thread
    pub fn init_pipeline(glfw: &mut glfw::Glfw, window: &mut glfw::Window) {
        unsafe {
            render_pipeline.init_pipeline(glfw, window);
        }
    }
    // Wrapped functions so we can affect this static mut variable
    pub fn gen_texture(texture: Texture) -> GPUObject {
        unsafe {
            let x = render_pipeline.task_immediate(RenderTask::GenerateTexture(SharedGPUObject::new(texture)));
            texture.id = x;
            x
        }
    }
    // Compile a subshader
    pub fn create_subshader(subshader: SubShader) -> GPUObject {
        unsafe {
            let x = render_pipeline.task_immediate(RenderTask::CreateSubShader(SharedGPUObject::new(subshader)));
            subshader.id = x;
            x
        }
    }
}
