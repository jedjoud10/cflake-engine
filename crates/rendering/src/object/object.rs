use std::{marker::PhantomData, sync::atomic::AtomicPtr};

use others::CommandID;

// Trait that is implemented on PipelineObjects
pub trait PipelineObject {
    
}

// A GPU object, something that can be created using OpenGL, and that is also stored on the Render Thread
pub enum PipelineObjectTypes {

}