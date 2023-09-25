use crate::prelude::{Event, World};

/// A system is executed whenever something interesting happens
/// like an update event, tick event, or window event
/// Systems could be executed in parallel by making use of the "resources" type
/// to handle scheduling automatically
pub trait System<E: Event> {
    type Resources<'w>: 'w;

    /// Execute the system with the appropriate event context
    fn execute(&mut self, resources: &mut Self::Resources<'_>);
    
    /// Handle the order of system execution compared to other systems 
    fn inject(&mut self) -> InjectionOrder;
}

/// A sync system is like a normal system, but it is executed sequentially
/// This allows us to place "barriers" that execute some single threaded code on
/// the main thread before the parallel execution of systems
pub trait Barrier<E: Event> {
    /// Execute the barrier with the appropriate world
    fn execute(&mut self, world: &mut World);
    
    /// Handle the order of system execution compared to other systems 
    fn inject(&mut self) -> InjectionOrder;
}

/// How this system is going to execute in relation to other systems
/// This allows us to set dependencies, dependants, or inject systems within both
/// using "rules" that define what must execute before a system and after a system
pub struct InjectionOrder {
}