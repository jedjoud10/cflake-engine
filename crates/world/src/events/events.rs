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

    fn call<'a, 'p>(boxed: &mut Box<Self::DynFn>, args: &mut Self::Args<'a, 'p>)
    where
        'a: 'p;
}