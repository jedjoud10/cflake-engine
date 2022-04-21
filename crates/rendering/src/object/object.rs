use crate::pipeline::{Handle, Pipeline};

// An OpenGL trait that will be added to all the objects that actually create OpenGL objects upon their creation
// This also executes some drop code that will dispose of the OpenGL memory that we have allocated
pub trait Object
where
    Self: Sized + 'static,
{
    // Called before the object gets added to the collection
    // Useful when we want to initialize some OpenGL state or smthing like that
    fn init(&mut self, pipeline: &mut Pipeline) {}

    // Called when the object has 0 strong handles, thus it gets destroyed
    fn disposed(self) {}
}
