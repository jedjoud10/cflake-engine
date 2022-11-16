use crate::{Init, Registry, Shutdown, Update};

use winit::event::{DeviceEvent, WindowEvent};

// An event is something that can be stored within a Registry and can be called
// Events of the same type get all executed at the same time
// F: Fn(&mut World, &mut WindowEvent)
pub trait Event<C: Caller, ID> {
    type Args<'a, 'p>
    where
        'a: 'p;
    fn boxed(self) -> Box<C::DynFn>;
}

// Callers are trait wrappers around events that allows to use registries
// WindowEvent<'_>
pub trait Caller: 'static + Sized {
    type Args<'a, 'p>
    where
        'a: 'p;
    type DynFn: ?Sized + 'static;

    // Note for future self: Implemented this because having the user have the ability write
    // their own events is completely useless since they cannot call them anyways
    fn registry(events: &Events) -> &Registry<Self>;
    fn registry_mut(events: &mut Events) -> &mut Registry<Self>;

    fn call<'a, 'p>(boxed: &mut Box<Self::DynFn>, args: &mut Self::Args<'a, 'p>)
    where
        'a: 'p;
}

// This is the main event struct that contains all the registries
// We store all the registries in their own boxed type, but they can be casted to using Any
pub struct Events {
    pub(crate) window: Registry<WindowEvent<'static>>,
    pub(crate) device: Registry<DeviceEvent>,
    pub(crate) init: Registry<Init>,
    pub(crate) update: Registry<Update>,
    pub(crate) shutdown: Registry<Shutdown>,
}

impl Events {
    // Get a specific registry mutably using it's unique caller
    pub fn registry_mut<C: Caller>(&mut self) -> &mut Registry<C> {
        C::registry_mut(self)
    }

    // Get a specific registry immutably using it's unique caller
    pub fn registry<C: Caller>(&self) -> &Registry<C> {
        C::registry(self)
    }
}
