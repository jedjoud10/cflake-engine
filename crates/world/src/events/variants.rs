use std::marker::PhantomData;

use crate::{Event, Events, Registry, World, Caller, StageKey};
use glutin::{
    event::{DeviceEvent, WindowEvent},
    event_loop::EventLoop,
};

// Device event called when there is a new device / device change
impl Caller for DeviceEvent {
    type DynFn = dyn FnMut(&mut World, &DeviceEvent);

    fn registry(events: &Events) -> &Registry<Self> {
        &events.device
    }

    fn registry_mut(events: &mut Events) -> &mut Registry<Self> {
        &mut events.device
    }
}

impl<F: FnMut(&mut World, &DeviceEvent) + 'static> Event<DeviceEvent, ()> for F {
    type Args<'a, 'p> = (&'p mut World, &'p DeviceEvent) where 'a: 'p;

    fn call<'a, 'p>(boxed: &mut Box<<DeviceEvent as Caller>::DynFn>, args: &mut Self::Args<'a, 'p>) where 'a: 'p {
        boxed(&mut args.0, &args.1);
    }

    fn boxed(self) -> Box<<DeviceEvent as Caller>::DynFn> {
        Box::new(self)
    }
}

// Window event called when there is a change that occured to the window
impl Caller for WindowEvent<'static> {
    type DynFn = dyn FnMut(&mut World, &mut WindowEvent<'_>);

    fn registry(events: &Events) -> &Registry<Self> {
        &events.window
    }

    fn registry_mut(events: &mut Events) -> &mut Registry<Self> {
        &mut events.window
    }
}

impl<F: FnMut(&mut World, &mut WindowEvent<'_>) + 'static> Event<WindowEvent<'static>, ()> for F {
    type Args<'a, 'p> = (&'p mut World, &'p mut WindowEvent<'a>) where 'a: 'p;

    fn call<'a, 'p>(boxed: &mut Box<<WindowEvent<'static> as Caller>::DynFn>, args: &mut Self::Args<'a, 'p>) where 'a: 'p {
        boxed(&mut args.0, &mut args.1);
    }

    fn boxed(self) -> Box<<WindowEvent<'static> as Caller>::DynFn> {
        Box::new(self)
    }
}

// Init event marker(FnOnce, called at the start of the engine)
pub struct Init(());

impl Caller for Init {
    type DynFn = dyn FnOnce(&mut World, &EventLoop<()>);

    fn registry(events: &Events) -> &Registry<Self> {
        &events.init
    }

    fn registry_mut(events: &mut Events) -> &mut Registry<Self> {
        &mut events.init
    }
}

impl<F: FnOnce(&mut World) + 'static> Event<Init, ()> for F {
    type Args<'a, 'p> = (&'p mut World, &'p EventLoop<()>) where 'a: 'p;

    fn call<'a, 'p>(boxed: &mut Box<<Init as Caller>::DynFn>, args: &mut Self::Args<'a, 'p>) where 'a: 'p {
        let boxed = std::mem::replace(boxed, Box::new(|_, _| {}));
        boxed(args.0, args.1)
    }

    fn boxed(self) -> Box<<Init as Caller>::DynFn> {
        Box::new(|world: &mut World, _| {
            self(world);
        })
    }
}

impl<F: FnOnce(&mut World, &EventLoop<()>) + 'static> Event<Init, (&mut World, &EventLoop<()>)> for F {
    type Args<'a, 'p> = (&'p mut World, &'p EventLoop<()>) where 'a: 'p;

    fn call<'a, 'p>(boxed: &mut Box<<Init as Caller>::DynFn>, args: &mut Self::Args<'a, 'p>) where 'a: 'p {
        let boxed = std::mem::replace(boxed, Box::new(|_, _| {}));
        boxed(args.0, args.1)
    }

    fn boxed(self) -> Box<<Init as Caller>::DynFn> {
        Box::new(self)
    }
}

// Update event marker (called each frame)
pub struct Update(());

impl Caller for Update {
    type DynFn = dyn FnMut(&mut World);

    fn registry(events: &Events) -> &Registry<Self> {
        &events.update
    }

    fn registry_mut(events: &mut Events) -> &mut Registry<Self> {
        &mut events.update
    }
}

impl<F: FnMut(&mut World) + 'static> Event<Update, ()> for F {
    type Args<'a, 'p> = &'p mut World where 'a: 'p;

    fn call<'a, 'p>(boxed: &mut Box<<Update as Caller>::DynFn>, args: &mut Self::Args<'a, 'p>) where 'a: 'p {
        boxed(args)
    }

    fn boxed(self) -> Box<<Update as Caller>::DynFn> {
        Box::new(self)
    }
}

// Exit event marker (called at the end of the game)
pub struct Exit(());

impl Caller for Exit {
    type DynFn = dyn FnMut(&mut World);

    fn registry(events: &Events) -> &Registry<Self> {
        &events.exit
    }

    fn registry_mut(events: &mut Events) -> &mut Registry<Self> {
        &mut events.exit
    }
}

impl<F: FnMut(&mut World) + 'static> Event<Exit, ()> for F {
    type Args<'a, 'p> = &'p mut World where 'a: 'p;

    fn call<'a, 'p>(boxed: &mut Box<<Exit as Caller>::DynFn>, args: &mut Self::Args<'a, 'p>) where 'a: 'p {
        boxed(args)
    }

    fn boxed(self) -> Box<<Exit as Caller>::DynFn> {
        Box::new(self)
    }
}