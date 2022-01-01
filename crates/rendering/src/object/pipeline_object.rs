use std::marker::PhantomData;

// A simple pipeline object, stored on the Render thread
pub enum PipelineObject {

}
// A GPU object, something that can be created using OpenGL, and that is also stored on the Render Thread
pub trait GPUObject {
}

// An ID for the PipelineObject
pub struct PipelineObjectID {
}


// A simple ptr to the actual PipelineObjectID
pub struct AsyncPipelineObjectID<T>
    where T: GPUObject

{
    phantom: PhantomData<T>,
    // Contains the CommandID and a ptr to the PipelineObjectID
    
}


// A simple struct telling us if we have finished executing a specific command
pub struct AsyncPipelineCommand {
    // Contains the CommandID
}