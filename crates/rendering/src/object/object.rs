use crate::pipeline::{PipelineCollection, Handle};

// An OpenGLInitializer trait that will be added to all the objects that actually create OpenGL objects upon their creation
pub(crate) trait OpenGLInitializer {
    // Called whenever the element is added in it's corresponding PipelineCollection
    fn added(&mut self, collection: &mut PipelineCollection<Self>, handle: Handle<T>);
}