use crate::{Init, Registry, Update};
use glutin::event::{DeviceEvent, WindowEvent};

// An event is something that can be stored within a Registry and can be called
// Events of the same type get all executed at the same time
// F: Fn(&mut World, &mut WindowEvent)
pub trait Event<'a, C: Caller> {
    type Args<'p> where 'a: 'p;
    fn call<'p>(&mut self, args: &mut Self::Args<'p>) where 'a: 'p;
}

// Callers are trait wrappers around events that allows to use registries
// WindowEvent<'_>
pub trait Caller: 'static {    }


// Implemented for any type of Vector that contains (StageKey, Event)
pub trait RegistryVec<C: Caller> {

}


// This is the main event struct that contains all the registries
// We store all the registries in their own boxed type, but they can be casted to using Any
pub struct Events {
}

impl Events {
    /*
    // Get the registry of a specific descriptor from within the global events
    // This is the only way we can interface with the values stored within the event manager
    pub fn registry<M: Descriptor>(&mut self) -> &mut Registry<M> {
        M::registry(self)
    }

    // This will execute the events of a specific type
    // I cannot have this function inside the Registry since we have lifetime issue
    pub fn execute<'p, M: Descriptor + Caller>(&mut self, params: M::Params<'_>) {
        M::call(self, params)
    }
    */
}
