use super::InjectionOrder;
use crate::prelude::{Event, World, WorldView};

/// A system is executed whenever something interesting happens
/// like an update event, tick event, or window event
/// Systems could be executed in parallel by making use of the "resources" type
/// to handle scheduling automatically
pub trait System<E: Event>: 'static + Send + Sync {
    /// List of resources that we can access in the world
    fn access(&mut self) -> Vec<()>;

    /// Execute the system with the appropriate event context
    fn execute(&mut self, view: &mut WorldView);

    /// Handle the order of system execution compared to other systems
    fn inject(&mut self) -> InjectionOrder<E>;
}

/*
/// A sync system is like a normal system, but it is executed sequentially, on the main thread
/// The good thing about sync systems is that they allow you to add/remove resources from the world, s
pub trait SyncSystem<E: Event> {
    /// Execute the barrier with the appropriate world
    fn execute(&mut self, world: &mut TryWorld);

    /// Handle the order of system execution compared to other systems
    fn inject(&mut self) -> InjectionOrder<E>;
}
*/