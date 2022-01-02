use std::marker::PhantomData;

use crate::{Texture, object::{AsyncPipelineObjectID, PipelineObject, GPUObject}};

// A simple builder that can be used to create Pipeline Objects
pub struct Builder<T> {
    phantom: PhantomData<T>
}

// This will create the AsyncPipelineObjectID and return it, while also send it to the render thread
// This is only available for GPUObjects, which are objects specifically created with OpenGL
impl<T> Builder<T>
    where T: GPUObject
{
    // Create the AsyncPipelineObjectID
    pub fn build(self) -> AsyncGPUObjectID<T> {
        todo!();
    }
}

// This will create the AsyncPipelineObjectID and return it, while also send it to the render thread
// This is only available for GPUObjects, which are objects specifically created with OpenGL


impl Builder<Texture> {

}