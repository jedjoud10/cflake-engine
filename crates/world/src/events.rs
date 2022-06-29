use std::any::{Any, TypeId};
use ahash::AHashMap;
use glutin::{event_loop::EventLoop, event::{WindowEvent, DeviceEvent}};
use crate::World;

// I wrote this code at 2:42 am.
// It looks like ass, I know, but it works
// I sold my sold my soul to the devil

// A registry is a specialized container that contains boxed functions/closures
// Registries are categorized by their marker type's dynamic type (that is also a descriptor)
pub struct Registry<'b, 'd, M: Descriptor<'d>>(&'b mut Vec<Box<M::DynFunc>>);

impl<'b, 'd, M: Descriptor<'d>> Registry<'b, 'd, M> {
    // Insert a new event closure/function into the registry
    pub fn insert<P>(&mut self, event: impl Event<'d, M, P>) {
        // We do a massive amount of trolling
        let boxed = event.boxed();
        self.0.push(boxed);
    }

    // Execute all the events that are stored inside this registry
    fn execute<'a>(self, params: <M as Caller<'d, 'a>>::Params) where M: Caller<'d, 'a> {
        M::call(self, params);
    }
}

// Descriptors simply tell us how we should box the function
pub trait Descriptor<'d>: Sized + 'd {
    // DynFunc which is the dynamic unsized value that we will box
    // Ex. dyn FnOnce()
    type DynFunc: ?Sized;

    // Get the appropirate registry from the main events
    fn registry<'b>(events: &'b mut Events) -> Registry<'b, 'd, Self>;
}

// Callers will be implemented for all marker types. This is what will execute the events specifically
pub trait Caller<'d, 'p>: Descriptor<'d> {
    // Parameters needed to execute the descriptor
    type Params: 'p;

    // Execute all the events that are contained from within the registry
    fn call(registry: Registry<'_, 'd, Self>, params: Self::Params);
}

// This trat will be implemented for closures that take in "P" arguments and that are used by the "M" marker descriptor
pub trait Event<'d, M: Descriptor<'d>, P> {
    // Box the underlying event into it's proper DynFn dynamic trait object
    fn boxed(self) -> Box<M::DynFunc>;
}

// This is the main event struct that contains all the registries
// We store all the registries in their own boxed type, but they can be casted to using Any
pub struct Events {
    pub(crate) init: Vec<Box<<Init as Descriptor<'static>>::DynFunc>>,
    pub(crate) update: Vec<Box<<Update as Descriptor<'static>>::DynFunc>>,
    pub(crate) window: Vec<Box<<WindowEvent<'static> as Descriptor<'static>>::DynFunc>>,    
    pub(crate) device: Vec<Box<<DeviceEvent as Descriptor<'static>>::DynFunc>>, 
}

impl Events {
    // Get the registry of a specific descriptor from within the global events
    // This is the only way we can interface with the values stored within the event manager
    pub fn registry<'b, 'd, M: Descriptor<'d>>(&'b mut self) -> Registry<'b, 'd, M> {
        Descriptor::registry(self)
    }
}

// Init event marker(FnOnce, called at the start of the engine)
pub struct Init(());

impl Descriptor<'static> for Init {
    type DynFunc = dyn FnOnce(&mut World, &EventLoop<()>);

    fn registry<'b>(events: &'b mut Events) -> Registry<'b, 'static, Self> {
        Registry(&mut events.init)
    }
}

impl<'p> Caller<'static, 'p> for Init {
    type Params = (&'p mut World, &'p EventLoop<()>);

    fn call(registry: Registry<'_, 'static, Self>, params: Self::Params) {
        let world = params.0;
        let el = params.1;

        let vec = std::mem::take(registry.0);

        for boxed in vec {
            boxed(world, el);
        }
    }
}

impl<F: FnOnce(&mut World) + 'static> Event<'static, Init, &mut World> for F {
    fn boxed(self) -> Box<<Init as Descriptor<'static>>::DynFunc> {
        Box::new(|world, _| self(world))
    }
}

impl<F: FnOnce(&mut World, &EventLoop<()>) + 'static> Event<'static, Init, (&mut World, &EventLoop<()>)> for F {
    fn boxed(self) -> Box<<Init as Descriptor<'static>>::DynFunc> {
        Box::new(self)
    }
}

// Update event marker (called each frame)
pub struct Update(());

impl Descriptor<'static> for Update {
    type DynFunc = dyn Fn(&mut World);

    fn registry<'b>(events: &'b mut Events) -> Registry<'b, 'static, Self> {
        Registry(&mut events.update)
    }
}

impl<'p> Caller<'static, 'p> for Update {
    type Params = &'p mut World;

    fn call(registry: Registry<'_, 'static, Self>, params: Self::Params) {
        for boxed in registry.0.iter() {
            boxed(params);
        }
    }
}

impl<F: Fn(&mut World) + 'static> Event<'static, Update, &mut World> for F {
    fn boxed(self) -> Box<<Update as Descriptor<'static>>::DynFunc> {
        Box::new(move |world| self(world))
    }
}

// Window event marker (called by glutin handler) (this makes it extremely pain since the window event contains a lifetime)
impl<'d> Descriptor<'d> for WindowEvent<'d> {
    type DynFunc = dyn Fn(&mut World, &WindowEvent);

    fn registry<'b>(events: &'b mut Events) -> Registry<'b, 'd, Self> {
        Registry(&mut events.window)
    }
}

impl<'d, 'p> Caller<'d, 'p> for WindowEvent<'d> where 'd: 'p {
    type Params = (&'p mut World, &'p WindowEvent<'d>);

    fn call(registry: Registry<'_, 'd, Self>, params: Self::Params) {
        let world = params.0;
        let ev = params.1;

        for event in registry.0.iter() {
            event(world, ev);
        }
    }
}

impl<'d, F: Fn(&mut World, &WindowEvent<'_>) + 'static> Event<'d, WindowEvent<'d>, (&mut World, &WindowEvent<'_>)> for F {
    fn boxed(self) -> Box<<WindowEvent<'d> as Descriptor<'d>>::DynFunc> {
        Box::new(self)
    }
} 

// Device event marker (called by glutin handler)
impl Descriptor<'static> for DeviceEvent {
    type DynFunc = dyn Fn(&mut World, &DeviceEvent);

    fn registry<'b>(events: &'b mut Events) -> Registry<'b, 'static, Self> {
        Registry(&mut events.device)
    }
}

impl<'p> Caller<'static, 'p> for DeviceEvent {
    type Params = (&'p mut World, &'p DeviceEvent);

    fn call(registry: Registry<'_, 'static, Self>, params: Self::Params) {
        let world = params.0;
        let event = params.1;

        for boxed in registry.0.iter() {
            boxed(world, event);
        }
    }
}

impl<F: Fn(&mut World, &DeviceEvent) + 'static> Event<'static, DeviceEvent, (&mut World, &DeviceEvent)> for F {
    fn boxed(self) -> Box<<DeviceEvent as Descriptor<'static>>::DynFunc> {
        Box::new(move |world, event| self(world, event))
    }
}