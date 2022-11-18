

use crate::{Caller, Event, World};
use winit::{
    event::{DeviceEvent, WindowEvent},
    event_loop::EventLoop,
};

// Device event called when there is a new device / device change
impl Caller for DeviceEvent {
    type DynFn = dyn FnMut(&mut World, &DeviceEvent);
    type Args<'a, 'p> = (&'p mut World, &'p DeviceEvent) where 'a: 'p;

    fn call<'a, 'p>(boxed: &mut Box<<DeviceEvent as Caller>::DynFn>, args: &mut Self::Args<'a, 'p>)
    where
        'a: 'p,
    {
        boxed(args.0, args.1);
    }
}

impl<F: FnMut(&mut World, &DeviceEvent) + 'static> Event<DeviceEvent, (&mut World, &DeviceEvent)> for F {
    type Args<'a, 'p> = (&'p mut World, &'p DeviceEvent) where 'a: 'p;

    fn boxed(self) -> Box<<DeviceEvent as Caller>::DynFn> {
        Box::new(self)
    }
}

impl<F: FnMut() + 'static> Event<DeviceEvent, ()> for F {
    type Args<'a, 'p> = (&'p mut World, &'p DeviceEvent) where 'a: 'p;

    fn boxed(mut self) -> Box<<DeviceEvent as Caller>::DynFn> {
        Box::new(move |_, _| self())
    }
}

// Window event called when there is a change that occured to the window
impl Caller for WindowEvent<'static> {
    type DynFn = dyn FnMut(&mut World, &mut WindowEvent<'_>);
    type Args<'a, 'p> = (&'p mut World, &'p mut WindowEvent<'a>) where 'a: 'p;

    fn call<'a, 'p>(
        boxed: &mut Box<<WindowEvent<'static> as Caller>::DynFn>,
        args: &mut Self::Args<'a, 'p>,
    ) where
        'a: 'p,
    {
        boxed(args.0, args.1);
    }
}

impl<F: FnMut(&mut World, &mut WindowEvent<'_>) + 'static> Event<WindowEvent<'static>, (&mut World, &mut WindowEvent<'_>)> for F {
    type Args<'a, 'p> = (&'p mut World, &'p mut WindowEvent<'a>) where 'a: 'p;

    fn boxed(self) -> Box<<WindowEvent<'static> as Caller>::DynFn> {
        Box::new(self)
    }
}

impl<F: FnMut() + 'static> Event<WindowEvent<'static>, ()> for F {
    type Args<'a, 'p> = (&'p mut World, &'p mut WindowEvent<'a>) where 'a: 'p;

    fn boxed(mut self) -> Box<<WindowEvent<'static> as Caller>::DynFn> {
        Box::new(move |_, _| self())
    }
}

// Init event marker(FnOnce, called at the start of the engine)
pub struct Init(());

impl Caller for Init {
    type DynFn = dyn FnOnce(&mut World, &EventLoop<()>);
    type Args<'a, 'p> = (&'p mut World, &'p EventLoop<()>) where 'a: 'p;

    fn call<'a, 'p>(boxed: &mut Box<<Init as Caller>::DynFn>, args: &mut Self::Args<'a, 'p>)
    where
        'a: 'p,
    {
        let boxed = std::mem::replace(boxed, Box::new(|_, _| {}));
        boxed(args.0, args.1)
    }
}

impl<F: FnOnce(&mut World) + 'static> Event<Init, &mut World> for F {
    type Args<'a, 'p> = (&'p mut World, &'p EventLoop<()>) where 'a: 'p;

    fn boxed(self) -> Box<<Init as Caller>::DynFn> {
        Box::new(|world: &mut World, _| {
            self(world);
        })
    }
}

impl<F: FnOnce(&mut World, &EventLoop<()>) + 'static> Event<Init, (&mut World, &EventLoop<()>)>
    for F
{
    type Args<'a, 'p> = (&'p mut World, &'p EventLoop<()>) where 'a: 'p;

    fn boxed(self) -> Box<<Init as Caller>::DynFn> {
        Box::new(self)
    }
}

impl<F: FnOnce() + 'static> Event<Init, ()> for F {
    type Args<'a, 'p> = (&'p mut World, &'p EventLoop<()>) where 'a: 'p;

    fn boxed(self) -> Box<<Init as Caller>::DynFn> {
        Box::new(move |_, _| self())
    }
}

// Update event marker (called each frame)
pub struct Update(());

impl Caller for Update {
    type DynFn = dyn FnMut(&mut World);
    type Args<'a, 'p> = &'p mut World where 'a: 'p;

    fn call<'a, 'p>(boxed: &mut Box<<Update as Caller>::DynFn>, args: &mut Self::Args<'a, 'p>)
    where
        'a: 'p,
    {
        boxed(args)
    }
}

impl<F: FnMut(&mut World) + 'static> Event<Update, &mut World> for F {
    type Args<'a, 'p> = &'p mut World where 'a: 'p;

    fn boxed(self) -> Box<<Update as Caller>::DynFn> {
        Box::new(self)
    }
}

impl<F: FnMut() + 'static> Event<Update, ()> for F {
    type Args<'a, 'p> = &'p mut World where 'a: 'p;

    fn boxed(mut self) -> Box<<Update as Caller>::DynFn> {
        Box::new(move |_| self())
    }
}

// Exit event marker (called at the end of the game)
pub struct Shutdown(());

impl Caller for Shutdown {
    type DynFn = dyn FnMut(&mut World);
    type Args<'a, 'p> = &'p mut World where 'a: 'p;

    fn call<'a, 'p>(boxed: &mut Box<<Shutdown as Caller>::DynFn>, args: &mut Self::Args<'a, 'p>)
    where
        'a: 'p,
    {
        boxed(args)
    }
}

impl<F: FnMut(&mut World) + 'static> Event<Shutdown, &mut World> for F {
    type Args<'a, 'p> = &'p mut World where 'a: 'p;

    fn boxed(self) -> Box<<Shutdown as Caller>::DynFn> {
        Box::new(self)
    }
}

impl<F: FnMut() + 'static> Event<Shutdown, ()> for F {
    type Args<'a, 'p> = &'p mut World where 'a: 'p;

    fn boxed(mut self) -> Box<<Shutdown as Caller>::DynFn> {
        Box::new(move |_| self())
    }
}