use crate::{Texture, Material, Buildable, Shader};
use super::{ObjectID, PipelineObject};

// A task to create an object
pub(crate) struct ObjectBuildingTask<T: PipelineObject + Buildable>(pub T, pub ObjectID<T>);
// A pipeline task that will be sent to the render thread
pub enum PipelineTask {
    CreateTexture(ObjectBuildingTask<Texture>),
    CreateMaterial(ObjectBuildingTask<Material>),
    CreateShader(ObjectBuildingTask<Shader>),
}

// The status for a specific PipelineTask
pub enum PipelineTaskStatus {
    Pending,
    Running,
    Finished,
}