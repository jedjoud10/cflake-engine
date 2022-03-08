use crate::{object::OpenGLObjectNotInitialized, pipeline::Pipeline};
// OpenGL buffer ops
pub trait GLBufferOperations
where
    Self: Sized,
{
    type Data;
    fn glset(&mut self, data: Self::Data) -> Result<(), OpenGLObjectNotInitialized>;
    fn glread(&mut self) -> Result<&Self::Data, OpenGLObjectNotInitialized>;
}
