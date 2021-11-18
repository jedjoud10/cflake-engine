use crate::RenderPipeline;
// Static mut RenderPipeline
pub static mut render_pipeline: RenderPipeline = RenderPipeline::default();

pub mod pipeline_commands {
    use crate::{render_pipeline, GPUObject, RenderTask, SubShader, Texture};
    use std::rc::Rc;
    // Wrapped functions so we can affect this static mut variable
    pub fn gen_texture(texture: Texture) -> GPUObject {
        unsafe {
            let x = render_pipeline.task_immediate(RenderTask::GenerateTexture(texture));
            x
        }
    }
    // Compile a subshader
    pub fn create_subshader(subshader: SubShader) -> GPUObject {
        unsafe {
            let x = render_pipeline.task_immediate(RenderTask::CreateSubShader(subshader));
            x
        }
    }
}
