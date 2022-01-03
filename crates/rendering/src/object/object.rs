use crate::PipelineObjectBuilder;
use super::PipelineTask;

// Trait that is implemented on PipelineObjects
pub trait PipelineObject {
    // Get a builder for this pipeline object, so we can actually create it
    fn builder() -> PipelineObjectBuilder<Self>;
}

// A GPU object, something that can be created using OpenGL, and that is also stored on the Render Thread
pub enum PipelineObjectTypes {

}