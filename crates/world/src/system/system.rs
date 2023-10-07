use std::any::TypeId;

use super::InjectionOrder;
use crate::{prelude::{Event, World}, resource::Resource};

/// A system is executed whenever something interesting happens
/// like an update event, tick event, or window event
/// Events are implemented for all function types like raw fns and closures,
/// but nothing would stop you from implementing it yourself on your own type
pub trait System<E: Event>: 'static {
    /// Execute the system with the given event type
    fn execute(&mut self, world: &mut World, e: &E);
}

impl<E: Event, F: FnMut(&mut World, &E) + 'static> System<E> for F {
    fn execute(&mut self, world: &mut World, e: &E) {
        (self)(world, e);
    }
}