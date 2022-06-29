use std::any::{Any, TypeId};

use ahash::AHashMap;

// A registry is a specialized container that contains boxed functions/closures
// Registries are categorized by their marker type (that is also a descriptor)
pub struct Registry<M: Descriptor>(Vec<Box<M::DynFunc>>);

impl<M: Descriptor> Registry<M> {
    // Insert a new event closure/function into the registry
    pub fn insert<P>(&mut self, event: impl Event<M, P>) {
        // We do a massive amount of trolling
        let boxed = event.boxed();
        self.0.push(boxed);
    }

    // Execute all the events that are stored inside this registry
    fn execute<'a>(&mut self, params: <M as Caller<'a>>::Params) where M: Caller<'a> {
        M::call(self, params);
    }
}

// Descriptors simply tell us how we should box the function
pub trait Descriptor: Sized + 'static {
    // DynFunc which is the dynamic unsized value that we will box
    // Ex. dyn FnOnce()
    type DynFunc: ?Sized;
}

// Callers will be implemented for all marker types. This is what will execute the events specifically
pub trait Caller<'a>: Descriptor {
    // Parameters needed to execute the descriptor
    type Params: 'a;

    // Execute all the events that are contained from within the registry
    fn call(registry: &mut Registry<Self>, params: Self::Params);
}

// This trat will be implemented for closures that take in "P" arguments and that are used by the "M" marker descriptor
pub trait Event<M: Descriptor, P> {
    // Box the underlying event into it's proper DynFn dynamic trait object
    fn boxed(self) -> Box<M::DynFunc>;
}

// This is the main event struct that contains all the registries
// We store all the registries in their own boxed type, but they can be casted to using Any
// The user should not be able to create their own events
pub struct Events(pub(crate) AHashMap<TypeId, Box<dyn Any>>);

impl Events {
    // Try to get a specific registry of a specific marker descriptor
    // This will automatically insert it if missing
    pub fn registry<M: Descriptor>(&mut self) -> &mut Registry<M> {
        let registry = self.0.entry(TypeId::of::<M>()).or_insert_with(|| Box::new(Registry::<M>(Vec::new())));
        registry.downcast_mut::<Registry<M>>().unwrap()
    }

    // This will try to call all the events that are stored from with a registry
    // This will do nothing if the registry is missing
    pub fn call<'a, M: Descriptor + Caller<'a>>(&mut self, params: <M as Caller<'a>>::Params) {
        if let Some(boxed) = self.0.get_mut(&TypeId::of::<M>()) {
            let registry = boxed.downcast_mut::<Registry<M>>().unwrap();
            registry.execute(params);
        }
    }
}