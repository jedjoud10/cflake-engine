pub(crate) mod private {
    use crate::pipeline::Pipeline;

    // Just cause I don't the user accessing the methods
    // TODO: FIX THIS SHITTTT
    pub trait ObjectSealed
    where
        Self: Sized + 'static,
    {
        // Called before the object gets added to the collection
        // Useful when we want to initialize some OpenGL state or smthing like that
        fn init(&mut self, _pipeline: &mut Pipeline) {}
        // Called when the object has 0 strong handles, thus it gets destroyed
        fn disposed(self) {}
    }
}
pub(crate) use private::ObjectSealed;
// An OpenGL trait that will be added to all the objects that actually create OpenGL objects upon their creation
// This also executes some drop code that will dispose of the OpenGL memory that we have allocated
pub trait Object: ObjectSealed
where
    Self: Sized + 'static,
{
}

impl<T: ObjectSealed> Object for T {}
