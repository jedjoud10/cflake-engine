use std::marker::PhantomData;

use others::TaskSender;

use crate::{Texture, object::{PipelineObjectID, PipelineObject, AsyncPipelineTaskID, PipelineTask}, SharedPipeline};

// A simple builder that can be used to create Pipeline Objects
pub struct PipelineObjectBuilder<T> 
    where T: PipelineObject
{
    pub(crate) data: T,
}

// This will create the PipelineObjectID and return it, while also send it to the render thread
// This is only available for GPUObjects, which are objects specifically created with OpenGL
impl<T> PipelineObjectBuilder<T>
    where T: PipelineObject, Self: BuilderConvert
{
    // Create a new builder using an already existing default value
    fn new(data: T) -> Self {
        Self {
            data
        }
    }
}

// Pub(Crate) trait
pub(crate) trait BuilderConvert {
    // Turn our raw PipelineObjectBuilder into it's creation task, so we can actually send it to the render thread
    fn convert(self) -> PipelineTask;    
}

// This is a task builer, and it will create tasks and send them to the render thread.
// This will also return an AsyncPipelineTaskID, which can be used to check whether the GPU task has finished executing or not
pub struct TaskBuilder {
}

impl TaskBuilder {
    // Build a task and send it to the render thread 
    pub fn build(task: PipelineTask, pipeline: &SharedPipeline) -> AsyncPipelineTaskID {        
    }
}

// This will create the AsyncPipelineObjectID and return it, while also send it to the render thread
// This is only available for GPUObjects, which are objects specifically created with OpenGL