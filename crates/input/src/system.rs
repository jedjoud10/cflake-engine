use crate::{Keyboard, Mouse};
use world::{DeviceEvent, Events, Init, Resource, World};

// This is the main input resource that will be stored persistently withint the world
// It allows us to get and modify it's underlying Keyboard and Mouse values
#[derive(Resource)]
pub struct Input(Keyboard, Mouse);

// This system will automatically insert the input resource and update it each frame using the device events
pub fn system(events: &mut Events) {
    // Init event (called during world init)
    events.register::<Init>(|world: &mut World| {
        world.insert(Input(
            Keyboard {
                binds: Default::default(),
                keys: Default::default(),
            },
            Mouse {
                scroll_delta: 0.0,
                scroll: 0.0,
                pos_delta: Default::default(),
                pos: Default::default(),
            },
        ))
    });

    // Device event (called when we receive a new device event from glutin)
    events.register::<DeviceEvent>(|world: &mut World, ev: &glutin::event::DeviceEvent| {});
}
