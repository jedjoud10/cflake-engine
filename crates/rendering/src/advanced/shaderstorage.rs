use crate::{object::{PipelineObject, ObjectID, PipelineTask, ObjectBuildingTask}, basics::Buildable};

// An OpenGL SSBO
#[derive(Default)]
pub struct ShaderStorage {
    // The OpenGL name for the underlying buffer
    pub(crate) oid: u32
    // How fre
}

impl PipelineObject for ShaderStorage {}

impl Buildable for ShaderStorage {
    fn construct_task(self, pipeline: &crate::pipeline::Pipeline) -> (crate::object::PipelineTask, crate::object::ObjectID<Self>) {
        // Create the ID
        let id = pipeline.atomics.get_next_id_increment();
        let id = ObjectID::new(id);
        // Create a task and send it
        (PipelineTask::CreateShaderStorage(ObjectBuildingTask::<Self>(self, id)), id)
    }
}