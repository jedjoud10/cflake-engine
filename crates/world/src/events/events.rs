use crate::{Init, Registry, StageKey, Update};
use glutin::event::{DeviceEvent, WindowEvent};

// Descriptors simply tell us how we should box the function
pub trait Descriptor: Sized {
    // DynFunc which is the dynamic unsized value that we will box
    // Ex. dyn FnOnce()
    type DynFunc: ?Sized;

    // This will fetch the appropriate registry for this specific marker from the main events
    fn registry(events: &mut Events) -> &mut Registry<Self>;
}

// Callers will be implemented for all marker types. This is what will execute the events specifically
pub trait Caller<'p>: Descriptor {
    // Parameters needed to execute the descriptor
    type Params: 'p;

    // Execute all the events that are contained from within the registry
    //fn call(registry: Registry<'d, Self>, params: Self::Params);
    fn call(vec: &mut Vec<(StageKey, Box<Self::DynFunc>)>, params: Self::Params);
}

// This trat will be implemented for closures that take in "P" arguments and that are used by the "M" marker descriptor
pub trait Event<M: Descriptor, P> {
    // Box the underlying event into it's proper DynFn dynamic trait object
    fn boxed(self) -> Box<M::DynFunc>;
}

// This is the main event struct that contains all the registries
// We store all the registries in their own boxed type, but they can be casted to using Any
pub struct Events {
    pub(crate) window: Registry<WindowEvent<'static>>,
    pub(crate) device: Registry<DeviceEvent>,
    pub(crate) init: Registry<Init>,
    pub(crate) update: Registry<Update>,
}

impl Events {
    // Get the registry of a specific descriptor from within the global events
    // This is the only way we can interface with the values stored within the event manager
    pub fn registry<M: Descriptor>(&mut self) -> &mut Registry<M> {
        M::registry(self)
    }
}
