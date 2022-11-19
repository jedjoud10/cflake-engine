use crate::{Caller, Event, Init, Registry, Rule, Shutdown, Update};
use ahash::AHashSet;
use std::{any::TypeId, marker::PhantomData};
use winit::event::{DeviceEvent, WindowEvent};

// Systems are collections of multiple events that we insert onto the world
// Systems can be added onto the current app using the insert method
// This system struct will only contain the event registries of all combined systems
pub struct Systems {
    pub(crate) hashset: AHashSet<TypeId>,
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
        let id = TypeId::of::<F>();
        if !self.hashset.contains(&id) {
            self.hashset.insert(id);
            let mut system = System {
                init: &mut self.init,
                update: &mut self.update,
                shutdown: &mut self.shutdown,
                window: &mut self.window,
                device: &mut self.device,
            };
            callback(&mut system);
        }
    }
}

// This is a mutable refernece to an event that was added to the system
// This allows us to specifiy the ordering of the specific event
pub struct EventMut<'a, C: Caller> {
    rules: &'a mut Vec<Rule>,
    default: bool,
    _phantom: PhantomData<C>,
}

impl<'a, C: Caller> EventMut<'a, C> {
    // Tell the event to execute before another event
    pub fn before<ID>(
        mut self,
        other: impl Event<C, ID> + 'static,
    ) -> Self {
        if self.default {
            self.rules.clear();
            self.default = false;
        }

        let (id, _) = super::id(other);
        self.rules.push(Rule::Before(id));
        self
    }

    // Tell the event to execute after another event
    pub fn after<ID>(
        mut self,
        other: impl Event<C, ID> + 'static,
    ) -> Self {
        if self.default {
            self.rules.clear();
            self.default = false;
        }

        let (id, _) = super::id(other);
        self.rules.push(Rule::After(id));
        self
    }
}

// This is a single system that will be passed along a callback
// This system is the main entry point to modify how events are executed
pub struct System<'a> {
    init: &'a mut Registry<Init>,
    update: &'a mut Registry<Update>,
    shutdown: &'a mut Registry<Shutdown>,
    window: &'a mut Registry<WindowEvent<'static>>,
    device: &'a mut Registry<DeviceEvent>,
}

macro_rules! insert {
    ($self:ident, $event:ident, $name:ident) => {{
        let rules = $self.$name.insert($event).unwrap();

        EventMut {
            rules,
            default: true,
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
        insert!(self, event, init)
    }

    // Insert an update event and return a mut event
    pub fn insert_update<ID>(
        &mut self,
        event: impl Event<Update, ID>,
    ) -> EventMut<Update> {
        insert!(self, event, update)
    }

    // Insert a shutdown event and return a mut event
    pub fn insert_shutdown<ID>(
        &mut self,
        event: impl Event<Shutdown, ID>,
    ) -> EventMut<Shutdown> {
        insert!(self, event, shutdown)
    }

    // Insert a device event and return a mut event
    pub fn insert_device<ID>(
        &mut self,
        event: impl Event<DeviceEvent, ID>,
    ) -> EventMut<DeviceEvent> {
        insert!(self, event, device)
    }

    // Insert a window event and return a mut event
    pub fn insert_window<ID>(
        &mut self,
        event: impl Event<WindowEvent<'static>, ID>,
    ) -> EventMut<WindowEvent<'static>> {
        insert!(self, event, window)
    }
}
