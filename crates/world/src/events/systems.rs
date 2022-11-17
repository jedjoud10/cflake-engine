use std::{rc::Rc, marker::PhantomData, any::{TypeId, Any}};

use ahash::AHashMap;
use winit::event::{DeviceEvent, WindowEvent};
use crate::{Init, Shutdown, Update, Caller, Event, Registry, Rule};


// Systems are collections of multiple events that we insert onto the world
// Systems can be added onto the current app using the insert method
// This system struct will only contain the event registries of all combined systems
pub struct Systems {
    pub(crate) init: Registry<Init>,
    pub(crate) update: Registry<Update>,
    pub(crate) shutdown: Registry<Shutdown>,
    pub(crate) window: Registry<WindowEvent<'static>>,
    pub(crate) device: Registry<DeviceEvent>,
}


// This is a mutable refernece to an event that was added to the system
// This allows us to specifiy the ordering of the specific event
pub struct EventMut<'a, C: Caller> {
    rules: &'a mut Vec<Rule>, 
    default: bool,
    _phantom: PhantomData<C>
}

impl<'a, C: Caller> EventMut<'a, C> {
    // Tell the event to execute before another event
    pub fn before<ID>(mut self, other: impl Event<C, ID> + 'static) -> Self {
        if self.default {
            self.rules.clear();
            self.default = false;
        }

        let (id, _) = super::id(other);
        self.rules.push(Rule::Before(id));
        self
    }
    
    // Tell the event to execute after another event
    pub fn after<ID>(mut self, other: impl Event<C, ID> + 'static) -> Self {
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
    ($self:ident, $event:ident, $name:ident) => {
        {
            let boxed = $event.boxed();
            let rules = $self.$name.insert(boxed).unwrap();
            
            EventMut {
                rules,
                default: true,
                _phantom: PhantomData,
            }
        }
    };
}

impl<'a> System<'a> {
    // Insert an init event and return a mut event
    pub fn insert_init<ID>(&mut self, event: impl Event<Init, ID>) -> EventMut<Init> {
        insert!(self, event, init)
    }

    // Insert an update event and return a mut event
    pub fn insert_update<ID>(&mut self, event: impl Event<Update, ID>) -> EventMut<Update> {
        insert!(self, event, update)
    }
    
    // Insert a shutdown event and return a mut event
    pub fn insert_shutdown<ID>(&mut self, event: impl Event<Shutdown, ID>) -> EventMut<Shutdown> {
        insert!(self, event, shutdown)
    }

    // Insert a device event and return a mut event
    pub fn insert_device<ID>(&mut self, event: impl Event<DeviceEvent, ID>) -> EventMut<DeviceEvent> {
        insert!(self, event, device)
    }

    // Insert a window event and return a mut event
    pub fn insert_window<ID>(&mut self, event: impl Event<WindowEvent<'static>, ID>) -> EventMut<WindowEvent<'static>> {
        insert!(self, event, window)
    }
}