use std::marker::PhantomData;

use crate::{Texture, object::{PipelineObject, PipelineTask, ObjectID, TaskID}, Pipeline};
// A buildable trait that can be implemented on Pipeline Objects that can be constructed using crate::pipec::construct()
pub(crate) trait Buildable: PipelineObject
    where Self: Sized
{
    // This is the first and single time that we will have access to the rendering pipeline, so if we wish to set any defaults that are missing we can do so here
    // This is ran before we actually construct the object's ID, so we are fine
    fn pre_construct(self, pipeline: &Pipeline) -> Self { self }
    // Construct the ID for self, send a task to the pipeline to create "self", and return our ID
    fn construct(self, pipeline: &Pipeline) -> ObjectID<Self>;
    // Create self. This basically replaces the Default implementation, since we will use some sort of "self" builder
    fn new() -> Self;
}