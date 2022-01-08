use crate::{Texture, Material};
use super::ObjectID;

// A task to create an object
pub(crate) struct ObjectBuildingTask<T>(pub T, pub ObjectID<T>);
// A pipeline task that will be sent to the render thread
pub enum PipelineTask {
    CreateTexture(ObjectBuildingTask<Texture>),
    CreateMaterial(ObjectBuildingTask<Material>),
}

// The status for a specific PipelineTask
pub enum PipelineTaskStatus {
    Pending,
    Running,
    Finished,
}