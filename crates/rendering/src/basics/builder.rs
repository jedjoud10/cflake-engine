use std::marker::PhantomData;

use crate::{Texture, object::{PipelineObject, PipelineTask, ObjectID, TaskID}, Pipeline};
// A buildable trait that can be implemented on Pipeline Objects that can be constructed using crate::pipec::construct()
pub(crate) trait Buildable: PipelineObject
    where Self: Sized
{
    // Construct the ID for self, send a task to the pipeline to create "self", and return our ID
    fn construct(self, pipeline: &Pipeline) -> ObjectID<Self>;
    // Create self. This basically replaces the Default implementation because we also get access to the pipeline, which is better
    fn new(pipeline: &Pipeline) -> Self;
}