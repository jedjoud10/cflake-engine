use crate::events::Event;
use std::marker::PhantomData;

use super::System;

/// A registry is what will contain all the different stages, and their appropriate systems
/// Stages are executed sequentially, although the systems within them are executed in parallel (if possible)
// Each type of event contains one registry associated with it
pub struct Registry<E: Event> {
    systems: Vec<Box<dyn System<E>>>,
    _phantom: PhantomData<E>,
}

impl<E: Event> Registry<E> {
    /// Execute all the systems that are stored in this registry
    pub fn execute(&mut self, _args: E) {}
}
