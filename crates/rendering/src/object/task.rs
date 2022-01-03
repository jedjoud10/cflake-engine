use crate::{Texture, basics::PipelineObjectBuilder};

// A pipeline task that will be sent to the render thread
pub enum PipelineTask {
    CreateTexture(PipelineObjectBuilder<Texture>),
    CreateMaterial(PipelineObjectBuilder<Material>),

}

// The status for a specific PipelineTask
pub enum PipelineTaskStatus {
    Pending,
    Running,
    Finished,
}