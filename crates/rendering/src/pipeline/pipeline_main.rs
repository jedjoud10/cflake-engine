use crate::RenderPipeline;
// Static mut RenderPipeline
pub static mut render_pipeline: RenderPipeline = RenderPipeline::default();

pub mod pipec {
    use crate::pipeline::object::*;
    use crate::{render_pipeline, RenderTask, Shader, SharedData, SubShader, Texture};
    // Start the render pipeline by initializing OpenGL on the new render thread
    pub fn init_pipeline(glfw: &mut glfw::Glfw, window: &mut glfw::Window) {
        unsafe {
            render_pipeline.init_pipeline(glfw, window);
        }
    }

    // Actual commands start here
    pub fn create_texture(texture: Texture) -> TextureGPUObject {
        unsafe {
            match render_pipeline.task_immediate(RenderTask::TextureCreate(SharedData::new(texture))) {
                GPUObject::Texture(x) => x,
                _ => panic!(),
            }
        }
    }
    pub fn create_subshader(subshader: SubShader) -> SubShaderGPUObject {
        unsafe {
            match render_pipeline.task_immediate(RenderTask::SubShaderCreate(SharedData::new(subshader))) {
                GPUObject::SubShader(x) => x,
                _ => panic!(),
            }
        }
    }
    pub fn create_shader(shader: Shader) -> ShaderGPUObject {
        unsafe {
            match render_pipeline.task_immediate(RenderTask::ShaderCreate(SharedData::new(shader))) {
                GPUObject::Shader(x) => x,
                _ => panic!(),
            }
        }
    }
    pub fn create_compute_shader(shader: Shader) -> ComputeShaderGPUObject {
        unsafe {
            match render_pipeline.task_immediate(RenderTask::ShaderCreate(SharedData::new(shader))) {
                GPUObject::ComputeShader(x) => x,
                _ => panic!(),
            }
        }
    }
    fn get_gpu_object(name: &str) -> GPUObject {
        unsafe {
            render_pipeline.get_gpu_object(name)
        }
    }
}
