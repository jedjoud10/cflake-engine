use std::any::TypeId;

use super::InjectionOrder;
use crate::{prelude::{Event, World, WorldView}, resource::Resource};

/// A system is executed whenever something interesting happens
/// like an update event, tick event, or window event
/// By default, systems are executed sequentially, though you can
/// override this behavior by implementing the ParSystem trait 
/// which allows your systems to execute in parallel
pub trait System<E: Event>: 'static + Send + Sync {
    /// Execute the system with the given event type
    fn execute_with_event(&mut self, view: &mut WorldView, e: E) {
        self.execute(view);
    }

    /// Execute the system without the event type
    /// This is wrapped in the execute_with_event function above
    fn execute(&mut self, view: &mut WorldView);

    /// Handle the order of system execution compared to other systems
    fn inject(&mut self) -> InjectionOrder<E> {
        InjectionOrder::default()
    }
}

/// The ParSystem trait allows systems to execute in parallel
/// This trait also defines the resources that will be used before hand
pub trait ParSystem<E: Event>: 'static + Send + Sync {
    /// How the system should be scheduled
    fn scheduling(&mut self) -> SystemScheduling;
}

/// Defines how a system should be scheduled and executed
/// Contains the resources that multi-threaded systems will access
#[derive(Default)]
pub struct SystemScheduling {
    resources: Vec<(TypeId, ResourceAccess)>,
}

impl SystemScheduling {
    /// Defines what we will do to a specific resource during the
    /// execution fo this system. Does nothing if the system is
    /// going to be executed sequentially
    pub fn with<T: Resource>(self, access: ResourceAccess) -> Self {
        self
    }
}

/// Resource access that depicts how we will use a specific resource during the exection of a system
/// If we use a resource in a way that isn't allowed by the access, the program will panic
pub enum ResourceAccess {
    /// Immutable access to the resource, only allowing you to read from it.
    /// Do not that there is a possibility that other systems are reading from this resource as well.
    Immutable,
    
    /// Mutable access to the resource, allowing you to read and write to it.
    /// The resource will only be accessed by one system at a time for mutable acces.
    Mutable,

    /// Allows you to add and remove the resource, and to make entries out of it
    Entry,
}