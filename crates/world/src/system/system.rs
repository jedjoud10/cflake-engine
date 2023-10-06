use std::any::TypeId;

use super::InjectionOrder;
use crate::{prelude::{Event, World}, resource::Resource};

/// A system is executed whenever something interesting happens
/// like an update event, tick event, or window event
pub trait System: 'static {
    type Event: Event;
    
    /// Execute the system with the given event type
    fn execute_with_event(&mut self, world: &mut World, e: &Self::Event) {
        self.execute(world);
    }

    /// Execute the system without the event type
    /// This is wrapped in the execute_with_event function above
    fn execute(&mut self, world: &mut World);

    /// Handle the order of system execution compared to other systems
    fn inject(&mut self) -> InjectionOrder<Self::Event> {
        InjectionOrder::default()
    }
}