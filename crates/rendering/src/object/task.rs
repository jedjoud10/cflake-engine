use crate::{Texture, Material};

// A pipeline task that will be sent to the render thread
pub enum PipelineTask {
    CreateTexture(Texture),
    CreateMaterial(Material),

}

// The status for a specific PipelineTask
pub enum PipelineTaskStatus {
    Pending,
    Running,
    Finished,
}