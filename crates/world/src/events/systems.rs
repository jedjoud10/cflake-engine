use crate::{
    Caller, CallerId, Event, Init, Registry, Rule, Shutdown,
    SystemId, Update,
};

use log_err::LogErrResult;
use std::marker::PhantomData;
use winit::event::{DeviceEvent, WindowEvent};

// Systems are collections of multiple events that we insert onto the world
// Systems can be added onto the current app using the insert method
// This system struct will only contain the event registries of all combined systems
pub struct Systems {
    // Keeps track of all the events
    pub init: Registry<Init>,
    pub update: Registry<Update>,
    pub shutdown: Registry<Shutdown>,
    pub window: Registry<WindowEvent<'static>>,
    pub device: Registry<DeviceEvent>,
}

impl Systems {
    // Add a system to the systems using a callback function
    // This will not add duplicate systems
    pub fn insert<F: FnOnce(&mut System) + 'static>(
        &mut self,
        callback: F,
    ) {
        // Create a system that will modify the registries
        let mut system = System {
            init: &mut self.init,
            update: &mut self.update,
            shutdown: &mut self.shutdown,
            window: &mut self.window,
            device: &mut self.device,
            system: super::fetch_system_id(&callback),
        };

        // This will run a function over the system that will mutate the registries
        // This will also keep track of the event stage IDs
        callback(&mut system);
    }
}

// This is a mutable refernece to an event that was added to the system
// This allows us to specifiy the ordering of the specific event
pub struct EventMut<'a, C: Caller> {
    rules: &'a mut Vec<Rule>,
    default: bool,
    caller: CallerId,
    _phantom: PhantomData<C>,
}

impl<'a, C: Caller> EventMut<'a, C> {
    // Tell the event to execute before another system's matching event
    pub fn before(
        mut self,
        other: impl FnOnce(&mut System) + 'static,
    ) -> Self {
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
    pub fn after(
        mut self,
        other: impl FnOnce(&mut System) + 'static,
    ) -> Self {
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
        let _t = Some(0u32);
        self
    }
}

// This is a single system that will be passed along a callback
// This system is the main entry point to modify how events are executed
// TODO: Implement fixed tick system?
pub struct System<'a> {
    init: &'a mut Registry<Init>,
    update: &'a mut Registry<Update>,
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
        let rules =
            registry.insert($event, $self.system).log_unwrap();

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
    pub fn insert_init<ID>(
        &mut self,
        event: impl Event<Init, ID>,
    ) -> EventMut<Init> {
        insert!(self, event, init, Init)
    }

    // Insert an update event and return a mut event
    pub fn insert_update<ID>(
        &mut self,
        event: impl Event<Update, ID>,
    ) -> EventMut<Update> {
        insert!(self, event, update, Update)
    }

    // Insert a shutdown event and return a mut event
    pub fn insert_shutdown<ID>(
        &mut self,
        event: impl Event<Shutdown, ID>,
    ) -> EventMut<Shutdown> {
        insert!(self, event, shutdown, Shutdown)
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
