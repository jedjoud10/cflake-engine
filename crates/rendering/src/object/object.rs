use std::{marker::PhantomData, sync::atomic::AtomicPtr};

use others::CommandID;

// A simple pipeline object, stored on the Render thread
pub enum PipelineObject {

}
// A GPU object, something that can be created using OpenGL, and that is also stored on the Render Thread
pub trait GPUObject {
}