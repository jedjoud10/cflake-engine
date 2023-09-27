use std::{any::TypeId, time::Duration, marker::PhantomData};
use ahash::AHashMap;
use petgraph::{visit::Topo, Graph};
use crate::events::Event;

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
    /// Sort all the systems stored in the registry using the stages
    pub fn sort(&mut self) {
    }

    /// Execute all the systems that are stored in this registry
    pub fn execute(&mut self, mut args: E) {
    }
}