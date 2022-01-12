use crate::{
    object::{ObjectID, PipelineObject, PipelineTask},
    Pipeline,
};
// A buildable trait that can be implemented on Pipeline Objects that can be constructed using crate::pipec::construct()
pub trait Buildable: PipelineObject
where
    Self: Sized,
{
    // This is the first and single time that we will have access to the rendering pipeline, so if we wish to set any defaults that are missing we can do so here
    // This is ran before we actually construct the object's ID, so we are fine
    fn pre_construct(self, _pipeline: &Pipeline) -> Self {
        self
    }
    // Construct the ID for self, send a task to the pipeline to create "self", and return our ID
    fn construct_task(self, pipeline: &Pipeline) -> (PipelineTask, ObjectID<Self>);
}
