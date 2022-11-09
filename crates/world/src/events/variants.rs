use std::marker::PhantomData;

use crate::{Event, Events, Registry, World, Caller, StageKey};
use glutin::{
    event::{DeviceEvent, WindowEvent},
    event_loop::EventLoop,
};

// Init event marker(FnOnce, called at the start of the engine)
pub struct Init(());

// Update event marker (called each frame)
pub struct Update(());

// Exit event marker (called at the end of the game)
pub struct Exit(());

impl Caller for WindowEvent<'static> {
    type DynFn = dyn FnMut(&mut World, &mut WindowEvent<'_>);

    fn registry(events: &mut Events) -> &mut Registry<Self> {
        &mut events.window
    }
}

impl<'a, F: FnMut(&mut World, &mut WindowEvent<'_>) + 'static> Event<'a, WindowEvent<'static>> for F {
    type Args<'p> = (&'p mut World, &'p mut WindowEvent<'a>) where 'a: 'p;

    fn call<'p>(boxed: &Box<<WindowEvent<'static> as Caller>::DynFn>, args: &mut Self::Args<'p>) where 'a: 'p {
        let desr = &**boxed;
        desr(&mut args.0, &mut args.1);
    }

    fn boxed(self) -> Box<<WindowEvent<'static> as Caller>::DynFn> {
        Box::new(self)
    }
}