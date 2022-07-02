use std::{rc::Rc, marker::PhantomData};

use ahash::AHashMap;
use glutin::event::WindowEvent;
use crate::{Pipeline, StageKey};

pub trait IntoEntry<'d>: Descriptor + 'd {
    fn into_registry<'b>(events: &'b mut Events) -> RegistryEntry<'b, 'd, Self>;
}

// Descriptors simply tell us how we should box the function
pub trait Descriptor: Sized {
    // DynFunc which is the dynamic unsized value that we will box
    // Ex. dyn FnOnce()
    type DynFunc: ?Sized;
}

// Callers will be implemented for all marker types. This is what will execute the events specifically
pub trait Caller<'p>: Descriptor {
    // Parameters needed to execute the descriptor
    type Params: 'p;

    // Execute all the events that are contained from within the registry
    //fn call(registry: Registry<'d, Self>, params: Self::Params);
    fn call(ptrs: &mut Vec<(StageKey, Box<Self::DynFunc>)>, params: Self::Params);
}

// This trat will be implemented for closures that take in "P" arguments and that are used by the "M" marker descriptor
pub trait Event<M: Descriptor, P> {
    // Box the underlying event into it's proper DynFn dynamic trait object
    fn boxed(self) -> Box<M::DynFunc>;
}

// Registries are a way for us to interract with the events that are stored in the main event struct
// There is a fixed set of registries that are stored from within the main event set
pub struct RegistryEntry<'b, 'd, D: Descriptor + 'd + IntoEntry<'d>> {
    pub(crate) container: &'b mut AHashMap<&'static str, Pipeline<D>>,
    pub(crate) _phantom: PhantomData<&'d D>,
}

pub struct Container<D: Descriptor + IntoEntry<'static>> {
    pub(crate) map: AHashMap<&'static str, Pipeline<D>>,
}


// This is the main event struct that contains all the registries
// We store all the registries in their own boxed type, but they can be casted to using Any
pub struct Events {
    pub(crate) window: Container<WindowEvent<'static>>,
}