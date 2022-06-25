use crate::{Descriptor, BoxedEvent, World, Event
};


// This event is called only at the start of the engine
pub struct Init(());

impl<'a> Descriptor<'a> for Init {
    type Params = &'a mut World;
}

impl<'a, F: Fn(&mut World) + 'static> Event<'a, F> for Init {
    fn execute(func: &F, params: &mut Self::Params) {
        let world: &mut World = params;
        func(world);
    }
}

impl<F> BoxedEvent<Init> for F where F: Fn(&mut World) + 'static {
    fn execute<'a>(&self, params: &mut <Init as Descriptor<'a>>::Params) where Init: Descriptor<'a> {
        <Init as Event<'a, F>>::execute(self, params)
    }
}

// This event is called each frame
pub struct Update(());

impl<'a> Descriptor<'a> for Update {
    type Params = &'a mut World;
}

impl<'a, F: Fn(&mut World) + 'static> Event<'a, F> for Update {
    fn execute(func: &F, params: &mut Self::Params) {
        let world: &mut World = params;
        func(world);
    }
}

impl<F> BoxedEvent<Update> for F where F: Fn(&mut World) + 'static {
    fn execute<'a>(&self, params: &mut <Update as Descriptor<'a>>::Params) where Update: Descriptor<'a> {
        <Update as Event<'a, F>>::execute(self, params)
    }
}

// Called whenever we receive a new window event from glutin
pub struct WindowEvent(());

impl<'a> Descriptor<'a> for WindowEvent {
    type Params = (&'a mut World, &'a glutin::event::WindowEvent<'a>);
}

impl<'a, F: Fn(&mut World, &glutin::event::WindowEvent) + 'static> Event<'a, F> for WindowEvent {
    fn execute(func: &F, params: &mut Self::Params) {
        let world: &mut World = params.0;
        let ev: &glutin::event::WindowEvent = params.1;
        func(world, &ev);
    }
}

impl<F> BoxedEvent<WindowEvent> for F where F: Fn(&mut World, &glutin::event::WindowEvent) + 'static {
    fn execute<'a>(&self, params: &mut <WindowEvent as Descriptor<'a>>::Params) where WindowEvent: Descriptor<'a> {
        <WindowEvent as Event<'a, F>>::execute(self, params)
    }
}

// Called whenever we receive a new device event from glutin
pub struct DeviceEvent(());

impl<'a> Descriptor<'a> for DeviceEvent {
    type Params = (&'a mut World, &'a glutin::event::DeviceEvent);
}

impl<'a, F: Fn(&mut World, &glutin::event::DeviceEvent) + 'static> Event<'a, F> for DeviceEvent {
    fn execute(func: &F, params: &mut Self::Params) {
        let world: &mut World = params.0;
        let ev: &glutin::event::DeviceEvent = params.1;
        func(world, &ev);
    }
}

impl<F> BoxedEvent<DeviceEvent> for F where F: Fn(&mut World, &glutin::event::DeviceEvent) + 'static {
    fn execute<'a>(&self, params: &mut <DeviceEvent as Descriptor<'a>>::Params) where DeviceEvent: Descriptor<'a> {
        <DeviceEvent as Event<'a, F>>::execute(self, params)
    }
}