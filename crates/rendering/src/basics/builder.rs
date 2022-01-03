use std::marker::PhantomData;

use others::TaskSender;

use crate::{Texture, object::{AsyncPipelineObjectID, PipelineObject, AsyncPipelineTaskID, PipelineTask}, SharedPipeline};

// A simple builder that can be used to create Pipeline Objects
pub struct Builder<T> {
    phantom: PhantomData<T>
}

// This will create the AsyncPipelineObjectID and return it, while also send it to the render thread
// This is only available for GPUObjects, which are objects specifically created with OpenGL
impl<T> Builder<T>
    where T: PipelineObject
{
    // Create the AsyncPipelineObjectID
    pub fn build(self, context: &SharedPipeline, task_sender: &TaskSender<PipelineTask>) -> AsyncPipelineObjectID<T> {
        todo!();
    }
}

// This is a task builer, and it will create tasks and send them to the render thread.
// This will also return an AsyncPipelineTaskID, which can be used to check whether the GPU task has finished executing or not
pub struct TaskBuilder {
}

impl TaskBuilder {
    // Build a task and send it to the render thread 
    pub fn build(task: PipelineTask, context: &SharedPipeline) -> AsyncPipelineTaskID {
        
    }
}

// This will create the AsyncPipelineObjectID and return it, while also send it to the render thread
// This is only available for GPUObjects, which are objects specifically created with OpenGL


impl Builder<Texture> {

}