use std::any::TypeId;

use super::InjectionOrder;
use crate::{prelude::{Event, World, WorldView}, resource::Resource};

/// A system is executed whenever something interesting happens
/// like an update event, tick event, or window event
pub trait System: 'static {
    type Event: Event;
    
    /// Execute the system with the given event type
    fn execute_with_event(&mut self, view: &mut WorldView, e: &Self::Event) {
        self.execute(view);
    }

    /// Execute the system without the event type
    /// This is wrapped in the execute_with_event function above
    fn execute(&mut self, view: &mut WorldView);

    /// Handle the order of system execution compared to other systems
    fn inject(&mut self) -> InjectionOrder<Self::Event> {
        InjectionOrder::default()
    }
}