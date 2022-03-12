use crate::pipeline::Handle;

// An OpenGL trait that will be added to all the objects that actually create OpenGL objects upon their creation
// This also executes some drop code that will dispose of the OpenGL memory that we have allocated
pub trait PipelineCollectionElement
where
    Self: Sized,
{
    // Called whenever the element is added in it's corresponding PipelineCollection
    fn added(&mut self, handle: &Handle<Self>);
    fn disposed(self);
}
