use std::any::Any;

/// A resource is a global data type that will be stored within the world for the duration of the program
/// Resources can be shared amongst systems, thus allowing us to share data between ECS systems
pub trait Resource: 'static {
    /// Convert self to dyn Any
    fn as_any(&self) -> &dyn Any;

    /// Convert self to &mut dyn Any
    fn as_any_mut(&mut self) -> &mut dyn Any;

    /// Convert a Boxed resource into a Boxed trait object of the Any trait
    fn into_any(self: Box<Self>) -> Box<dyn Any>;
}

/// This is the main world state that the user can manually update to force the engine to stop running
#[derive(Default, Clone, Copy, PartialEq, Eq)]
pub enum State {
    #[default]
    /// This is the default state from frame 1 to frame n
    Running,

    /// This is only set manually, by the user
    Stopped,
}

impl<T: 'static> Resource for T {
    fn as_any(&self) -> &dyn Any {
        self
    }

    fn as_any_mut(&mut self) -> &mut dyn Any {
        self
    }

    fn into_any(self: Box<Self>) -> Box<dyn Any> {
        self
    }
}
