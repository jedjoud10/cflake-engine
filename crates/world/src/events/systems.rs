use crate::{Caller, CallerId, Event, Init, Registry, Rule, Shutdown, SystemId, Tick, Update};

use log_err::LogErrResult;
use std::marker::PhantomData;
use winit::event::{DeviceEvent, WindowEvent};

/// Systems are collections of multiple events that we insert onto the world
/// Systems can be added onto the current app using the insert method
/// This system struct will only contain the event registries of all combined systems
pub struct Systems {
    // Keeps track of all the events
    pub init: Registry<Init>,
    pub update: Registry<Update>,
    pub shutdown: Registry<Shutdown>,
    pub tick: Registry<Tick>,
    pub window: Registry<WindowEvent<'static>>,
    pub device: Registry<DeviceEvent>,
}

impl<'a, C: Caller> EventMut<'a, C> {
    // Tell the event to execute before another system's matching event
    pub fn before(mut self, other: impl FnOnce(&mut System) + 'static) -> Self {
        if self.default {
            self.rules.clear();
            self.default = false;
        }

        // Get the stage ID of the other system's event
        let system = super::fetch_system_id(&other);
        let stage = super::combine_ids(&system, &self.caller);

        // Create a rule based on that ID
        let rule = Rule::Before(stage);

        // Insert the rule internally
        self.rules.push(rule);
        self
    }

    // Tell the event to execute after another system's matching event
    pub fn after(mut self, other: impl FnOnce(&mut System) + 'static) -> Self {
        if self.default {
            self.rules.clear();
            self.default = false;
        }

        // Get the stage ID of the other system's event
        let system = super::fetch_system_id(&other);
        let stage = super::combine_ids(&system, &self.caller);

        // Create a rule based on that ID
        let rule = Rule::After(stage);

        // Insert the rule internally
        self.rules.push(rule);
        self
    }
}

// This is a single system that will be passed along a callback
// This system is the main entry point to modify how events are executed
pub struct System<'a> {
    init: &'a mut Registry<Init>,
    update: &'a mut Registry<Update>,
    tick: &'a mut Registry<Tick>,
    shutdown: &'a mut Registry<Shutdown>,
    window: &'a mut Registry<WindowEvent<'static>>,
    device: &'a mut Registry<DeviceEvent>,
    system: SystemId,
}

macro_rules! insert {
    ($self:ident, $event:ident, $name:ident, $C:ty) => {{
        // Get the correspodning registry
        let registry = &mut $self.$name;

        // Push the event into the registry
        let rules = registry.insert($event, $self.system).log_unwrap();

        // Create the caller ID
        let caller = super::fetch_caller_id::<$C>();

        EventMut {
            rules,
            default: true,
            caller,
            _phantom: PhantomData,
        }
    }};
}

impl<'a> System<'a> {
    // Insert an init event and return a mut event
    pub fn insert_init<ID>(&mut self, event: impl Event<Init, ID>) -> EventMut<Init> {
        insert!(self, event, init, Init)
    }

    // Insert an update event and return a mut event
    pub fn insert_update<ID>(&mut self, event: impl Event<Update, ID>) -> EventMut<Update> {
        insert!(self, event, update, Update)
    }

    // Insert a shutdown event and return a mut event
    pub fn insert_shutdown<ID>(&mut self, event: impl Event<Shutdown, ID>) -> EventMut<Shutdown> {
        insert!(self, event, shutdown, Shutdown)
    }

    // Insert a tick event and return a mut evnet
    pub fn insert_tick<ID>(&mut self, event: impl Event<Tick, ID>) -> EventMut<Tick> {
        insert!(self, event, tick, Tick)
    }

    // Insert a device event and return a mut event
    pub fn insert_device<ID>(
        &mut self,
        event: impl Event<DeviceEvent, ID>,
    ) -> EventMut<DeviceEvent> {
        insert!(self, event, device, DeviceEvent)
    }

    // Insert a window event and return a mut event
    pub fn insert_window<ID>(
        &mut self,
        event: impl Event<WindowEvent<'static>, ID>,
    ) -> EventMut<WindowEvent<'static>> {
        insert!(self, event, window, WindowEvent)
    }
}
