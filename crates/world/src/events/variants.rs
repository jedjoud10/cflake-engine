use crate::{Caller, Descriptor, Event, Events, Registry, World};
use glutin::{
    event::{DeviceEvent, WindowEvent},
    event_loop::EventLoop,
};

// Window event marker (called by glutin handler)
impl<'a> Descriptor for WindowEvent<'a> {
    type DynFunc = dyn Fn(&mut World, &mut WindowEvent);

    fn registry(events: &mut Events) -> &mut Registry<Self> {
        &mut events.window
    }
}

impl<'b, 'p> Caller<'p> for WindowEvent<'b>
where
    'b: 'p,
{
    type Params = (&'p mut World, &'p mut WindowEvent<'b>);

    fn call(events: &mut Events, params: Self::Params) {
        let world = params.0;
        let event = params.1;

        for (_, func) in events.window.events.iter() {
            func(world, event);
        }
    }
}

impl<'a, F: Fn(&mut World, &mut WindowEvent<'_>) + 'static>
    Event<WindowEvent<'a>, (&mut World, &mut WindowEvent<'_>)> for F
{
    fn boxed(self) -> Box<<WindowEvent<'a> as Descriptor>::DynFunc> {
        Box::new(self)
    }
}

impl<'a, F: Fn(&mut World, &WindowEvent<'_>) + 'static>
    Event<WindowEvent<'a>, (&mut World, &WindowEvent<'_>)> for F
{
    fn boxed(self) -> Box<<WindowEvent<'a> as Descriptor>::DynFunc> {
        Box::new(move |world, event| self(world, event))
    }
}

// Device event marker (called by glutin handler)
impl Descriptor for DeviceEvent {
    type DynFunc = dyn Fn(&mut World, &DeviceEvent);

    fn registry(events: &mut Events) -> &mut Registry<Self> {
        &mut events.device
    }
}

impl<'p> Caller<'p> for DeviceEvent {
    type Params = (&'p mut World, &'p DeviceEvent);

    fn call(events: &mut Events, params: Self::Params) {
        let world = params.0;
        let event = params.1;

        for (_, func) in events.device.events.iter() {
            func(world, event);
        }
    }
}

impl<F: Fn(&mut World, &DeviceEvent) + 'static> Event<DeviceEvent, (&mut World, &DeviceEvent)>
    for F
{
    fn boxed(self) -> Box<<DeviceEvent as Descriptor>::DynFunc> {
        Box::new(self)
    }
}

// Init event marker(FnOnce, called at the start of the engine)
pub struct Init(());

impl Descriptor for Init {
    type DynFunc = dyn FnOnce(&mut World, &EventLoop<()>);

    fn registry(events: &mut Events) -> &mut Registry<Self> {
        &mut events.init
    }
}

impl<'p> Caller<'p> for Init {
    type Params = (&'p mut World, &'p EventLoop<()>);

    fn call(events: &mut Events, params: Self::Params) {
        let world = params.0;
        let el = params.1;

        let take = std::mem::take(&mut events.init.events);

        for (_, func) in take {
            func(world, el)
        }
    }
}

impl<F: FnOnce(&mut World) + 'static> Event<Init, &mut World> for F {
    fn boxed(self) -> Box<<Init as Descriptor>::DynFunc> {
        Box::new(|world, _| self(world))
    }
}

impl<F: FnOnce(&mut World, &EventLoop<()>) + 'static> Event<Init, (&mut World, &EventLoop<()>)>
    for F
{
    fn boxed(self) -> Box<<Init as Descriptor>::DynFunc> {
        Box::new(self)
    }
}

// Update event marker (called each frame)
pub struct Update(());

impl Descriptor for Update {
    type DynFunc = dyn Fn(&mut World);

    fn registry(events: &mut Events) -> &mut Registry<Self> {
        &mut events.update
    }
}

impl<'p> Caller<'p> for Update {
    type Params = &'p mut World;

    fn call(events: &mut Events, params: Self::Params) {
        for (_, func) in events.update.events.iter() {
            func(params)
        }
    }
}

impl<F: Fn(&mut World) + 'static> Event<Update, &mut World> for F {
    fn boxed(self) -> Box<<Update as Descriptor>::DynFunc> {
        Box::new(self)
    }
}
