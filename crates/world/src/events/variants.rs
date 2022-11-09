use std::marker::PhantomData;

use crate::{Event, Events, Registry, World, Caller, RegistryVec, StageKey};
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
}

impl<'a, F: FnMut(&mut World, &mut WindowEvent<'_>)> Event<'a, WindowEvent<'static>> for F {
    type Args<'p> = (&'p mut World, &'p mut WindowEvent<'a>) where 'a: 'p;

    fn call<'p>(&mut self, args: &mut Self::Args<'p>) where 'a: 'p {
        let world = &mut args.0;
        let event = &mut args.1;
        self(world, event);
    }
}

impl<'a, C: Caller, E: Event<'a, C>> RegistryVec<C> for Vec<(StageKey, E)> {

} 