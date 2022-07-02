use crate::{Keyboard, Mouse};
use glutin::event::DeviceEvent;
use world::{Events, Init, Resource, Stage, World};

// This is the main input resource that will be stored persistently withint the world
// It allows us to get and modify it's underlying Keyboard and Mouse values
#[derive(Resource)]
pub struct Input(Keyboard, Mouse);

// This system will automatically insert the input resource and update it each frame using the device events
pub fn system(events: &mut Events) {
    // Init event (called once at the start of program)
    fn init(world: &mut World) {
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
    }

    // Glutin device event (called by handler when needed)
    fn event(_world: &mut World, _data: &DeviceEvent) {}

    // Register the events
    events.registry::<Init>().insert(init);
    events.registry::<DeviceEvent>().insert_with(
        event,
        Stage::new("input").before("internal begin"),
    );
}
