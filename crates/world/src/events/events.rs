use std::any::{TypeId, Any};

use crate::{Init, Registry, Update};
use ahash::AHashMap;
use glutin::event::{DeviceEvent, WindowEvent};

// An event is something that can be stored within a Registry and can be called
// Events of the same type get all executed at the same time
// F: Fn(&mut World, &mut WindowEvent)
pub trait Event<'a, C: Caller> {
    type Args<'p> where 'a: 'p;
    fn call<'p>(boxed: &Box<C::DynFn>, args: &mut Self::Args<'p>) where 'a: 'p;
    fn boxed(self) -> Box<C::DynFn>;
}

// Callers are trait wrappers around events that allows to use registries
// WindowEvent<'_>
pub trait Caller: 'static + Sized { 
    type DynFn: ?Sized + 'static;

    // Note for future self: Implemented this because having the user have the ability write 
    // their own events is completely useless since they cannot call them anyways
    fn registry(events: &mut Events) -> &mut Registry<Self>;
}


// This is the main event struct that contains all the registries
// We store all the registries in their own boxed type, but they can be casted to using Any
pub struct Events {
    pub(crate) window: Registry<WindowEvent<'static>>,
    /*
    pub(crate) device: Registry<DeviceEvent>,
    pub(crate) init: Registry<Init>,
    pub(crate) update: Registry<Update>,
    */
}

impl Events {
    // Get a specific registry mutably using it's unique caller
    pub fn registry_mut<C: Caller>(&mut self) -> &mut Registry<C> {
        C::registry(self)
    }
}
