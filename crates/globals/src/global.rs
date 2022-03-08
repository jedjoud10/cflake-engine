use std::any::Any;
// A global component that can only be accessed by the main thread
// This means that it doesn't need to be Sync
pub trait Global {
    fn as_any(&self) -> &dyn Any;
    fn as_any_mut(&mut self) -> &mut dyn Any;
}
pub type EnclosedGlobalComponent = Box<dyn Global>;
