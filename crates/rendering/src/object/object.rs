use crate::pipeline::{Handle, Pipeline};

// An OpenGL trait that will be added to all the objects that actually create OpenGL objects upon their creation
// This also executes some drop code that will dispose of the OpenGL memory that we have allocated
pub trait PipelineCollectionElement
where
    Self: Sized,
{
    // Called when we must add Self to the correct pipeline collection
    fn add(self, pipeline: &mut Pipeline) -> Handle<Self>;

    // Get, get_mut
    fn find<'a>(pipeline: &'a Pipeline, handle: &Handle<Self>) -> Option<&'a Self>;
    fn find_mut<'a>(pipeline: &'a mut Pipeline, handle: &Handle<Self>) -> Option<&'a mut Self>;

    // Called when the object has 0 strong handles, thus it gets destroyed
    fn disposed(self);
}
