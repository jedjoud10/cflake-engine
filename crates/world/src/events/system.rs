use std::rc::Rc;

use ahash::AHashMap;
use winit::event::{DeviceEvent, WindowEvent};
use crate::{Events, Init, Shutdown, Update, Caller, Stage, Event};

// Box wrapper for event
type E<C: Caller> = (Box<C::DynFn>, Stage<C>);

// Systems are collections of multiple events that we insert onto the world
// Systems can be added onto the current app using the insert_system method
pub struct System {
    pub(crate) window: Option<E<WindowEvent<'static>>>,
    pub(crate) device: Option<E<DeviceEvent>>,
    pub(crate) init: Option<E<Init>>,
    pub(crate) update: Option<E<Update>>,
    pub(crate) shutdown: Option<E<Shutdown>>,
}

// This trait is implemented for events that can be insterted inside systems
// It is implemented for normal event functions and tuples that contain a stage
// TODO: Rewrite this goofyy ahh code
pub trait SystemEvent<C: Caller, ID, F: Event<C, ID>>: Sized {
    fn convert(self) -> (F, Stage<C>);
}


impl<ID, F: Event<C, ID>, C: Caller> SystemEvent<C, ID, F> for F {
    fn convert(self) -> (F, Stage<C>) {
        let mut stage = Stage::new();
        let (id, event) = super::id(self);
        stage.rules = super::default_rules();
        stage.name = Some(id);
        (event, stage)
    }
}

impl<ID, F: Event<C, ID>, C: Caller> SystemEvent<C, ID, F> for (F, Stage<C>) {
    fn convert(self) -> (F, Stage<C>) {
        let mut stage = self.1;
        let (id, event) = super::id(self.0);
        stage.rules = super::default_rules();
        stage.name = Some(id);
        (event, stage)
    }
}

impl Default for System {
    fn default() -> Self {
        Self { 
            window: Default::default(),
            device: Default::default(),
            init: Default::default(),
            update: Default::default(),
            shutdown: Default::default()
        }
    }
}

impl System {   
    // Add an init event with implicit / explicit stage
    pub fn insert_init<ID, F: Event<Init, ID>>(mut self, event: impl SystemEvent<Init, ID, F>) -> Self {
        let (event, stage) = event.convert();
        self.init = Some((event.boxed(), stage));
        self
    }

    // Add an update event with implicit / explicit stage
    pub fn insert_update<ID, F: Event<Update, ID>>(mut self, event: impl SystemEvent<Update, ID, F>) -> Self {
        let (event, stage) = event.convert();
        self.update = Some((event.boxed(), stage));
        self
    }

    // Add a shutdown event with implicit / explicit stage
    pub fn insert_shutdown<ID, F: Event<Shutdown, ID>>(mut self, event: impl SystemEvent<Shutdown, ID, F>) -> Self {
        let (event, stage) = event.convert();
        self.shutdown = Some((event.boxed(), stage));
        self
    }
    
    // Add a window event with implicit / explicit stage
    pub fn insert_window<ID, F: Event<WindowEvent<'static>, ID>>(mut self, event: impl SystemEvent<WindowEvent<'static>, ID, F>) -> Self {
        let (event, stage) = event.convert();
        self.window = Some((event.boxed(), stage));
        self
    }
    
    // Add a device event with implicit / explicit stage
    pub fn insert_device<ID, F: Event<DeviceEvent, ID>>(mut self, event: impl SystemEvent<DeviceEvent, ID, F>) -> Self {
        let (event, stage) = event.convert();
        self.device = Some((event.boxed(), stage));
        self
    }
}