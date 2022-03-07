use crate::{object::OpenGLObjectNotInitialized, pipeline::Pipeline};
// Write the stored Rust data into the OpenGL buffer
pub trait Writable
where
    Self: Sized,
{
    fn glupdate(&mut self) -> Result<(), OpenGLObjectNotInitialized>;
}
// Read back the OpenGL data from the driver and insert it into the Rust
pub trait Readable
where
    Self: Sized,
{
    type Data;
    fn glread(&mut self) -> Result<&Self::Data, OpenGLObjectNotInitialized>;
}
