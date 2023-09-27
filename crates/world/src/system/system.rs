use crate::prelude::{Event, World};
use super::InjectionOrder;

/// A system is executed whenever something interesting happens
/// like an update event, tick event, or window event
/// Systems could be executed in parallel by making use of the "resources" type
/// to handle scheduling automatically
pub trait System<E: Event> {
    /// List of resources that we can access in the world
    fn access(&mut self) -> Vec<()>;

    /// Execute the system with the appropriate event context
    fn execute(&mut self, world: &mut World);
    
    /// Handle the order of system execution compared to other systems 
    fn inject(&mut self) -> InjectionOrder<E>;
}

/*
/// A sync system is like a normal system, but it is executed sequentially
/// This allows us to place "barriers" that execute some single threaded code on
/// the main thread before the parallel execution of systems
/// This doesn't require "access" as it could simply "fail" when there isn't a resource present in the world
pub trait Barrier<E: Event> {
    /// Execute the barrier with the appropriate world
    fn execute(&mut self, world: &mut TryWorld);
    
    /// Handle the order of system execution compared to other systems 
    fn inject(&mut self) -> InjectionOrder<E>;
}
*/