use crate::events::Event;
use std::marker::PhantomData;

/// A registry is what will contain all the different stages, and their appropriate systems
/// Stages are executed sequentially, although the systems within them are executed in parallel (if possible)
// Each type of event contains one registry associated with it
pub struct Registry<E: Event> {
    _phantom: PhantomData<E>,
}

impl<E: Event> Default for Registry<E> {
    fn default() -> Self {
        Self {
            _phantom: PhantomData,
        }
    }
}

impl<E: Event> Registry<E> {
    /// Sort all the systems stored in the registry using their given injection order
    pub fn sort(&mut self) {}

    /// Execute all the systems that are stored in this registry
    /// in parallel if possible
    pub fn execute(&mut self, _args: E) {}
}
