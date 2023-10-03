use std::marker::PhantomData;
use crate::events::Event;
use super::System;

/// A stage is a combination of systems that can execute in parallel
/// Systems stored within stages might read from multiple resources 
/// but they must write to unique resources that are not being currently read
/// by other systems
pub struct Stage<E: Event> {
    systems: Vec<Box<dyn System<Event = E>>>,
    _phantom: PhantomData<E>,
}