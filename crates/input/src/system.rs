use crate::{Keyboard, Mouse};
use world::{Events, Resource, World};

// This is the main input resource that will be stored persistently withint the world
// It allows us to get and modify it's underlying Keyboard and Mouse values
#[derive(Resource)]
#[Locked]
pub struct Input(Keyboard, Mouse);

impl Default for Input {
    fn default() -> Self {
        Input(
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
        )
    }
}

// This is the main system that we will insert in the app
pub fn system(events: &Events) {}
