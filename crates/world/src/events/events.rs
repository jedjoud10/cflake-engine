use ahash::AHashMap;
use glutin::event::WindowEvent;
use crate::{Pipeline, StageKey};

// Descriptors simply tell us how we should box the function
pub trait Descriptor: Sized {
    // DynFunc which is the dynamic unsized value that we will box
    // Ex. dyn FnOnce()
    type DynFunc: ?Sized;

    // Get the appropirate registry from the main events
    fn registry<'b>(events: &'b mut Events) -> &'b mut Registry<Self>;
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
pub struct Registry<D: Descriptor>(AHashMap<&'static str, Pipeline<D>>);

impl<D: Descriptor> Registry<D> {
    // Try to get a pipeline using it's name. If the pipeline does not exist, this will create it automatically
    pub fn pipeline(&mut self, name: &'static str) -> &mut Pipeline<D> {
        self.0.entry(name).or_insert_with(|| Pipeline {
            map: Default::default(),
            events: Default::default(),
        })
    }
}

// This is the main event struct that contains all the registries
// We store all the registries in their own boxed type, but they can be casted to using Any
pub struct Events {
    pub(crate) window: Registry<WindowEvent<'static>>,
}