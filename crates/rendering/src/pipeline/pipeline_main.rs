use crate::RenderPipeline;
// Static mut RenderPipeline
pub static mut render_pipeline: RenderPipeline = RenderPipeline::default();

pub mod pipeline_commands {
    use std::sync::Arc;
    use crate::{GPUObject, RenderTask, Texture, render_pipeline};    
    // Wrapped functions so we can affect this static mut variable
    pub fn gen_texture(texture: Arc<Texture>) -> GPUObject {
        unsafe {
            // Call the render pipeline function
            let x = render_pipeline.task_immediate(RenderTask::GenerateTexture(texture));
            x
        }
    }
}