use std::marker::PhantomData;

use crate::{Texture, object::{PipelineObject, PipelineTask, ObjectID, TaskID}, Pipeline};
// A buildable trait that can be implemented on Pipeline Objects that can be constructed using crate::pipec::construct()
pub(crate) trait Buildable: PipelineObject
    where Self: Sized
{
    // Create a task, send it, and return the Object ID for this specific PipelineObject
    fn send(self, pipeline: &Pipeline) -> ObjectID<Self>;
    // Create self. This basically replaces the Default implementation because we also get access to the pipeline, which is better
    fn new(pipeline: &Pipeline) -> Self;
}