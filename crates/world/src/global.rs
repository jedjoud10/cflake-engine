use std::sync::{
    atomic::{AtomicBool, Ordering},
    Mutex,
};

use crate::{Events, World};

// The setup function can only be called once
static INITIALIZED: AtomicBool = AtomicBool::new(false);

// This global function will be used to initialize the Events and the World
// This will be called by the main glutin handler, but it can only be called once
pub fn setup() -> (World, Events) {
    if INITIALIZED.load(Ordering::Relaxed) {
        (
            // Create a single instance of the world
            World {
                resources: Default::default(),
            },

            // Create a single instance of the events
            Events {
                init: Default::default(),
                update: Default::default(),
                window: Default::default(),
                device: Default::default(),
                glutin_init: Default::default(),
            },
        )
    } else {
        // We've already created the world and event manager, so we must panic
        panic!("Cannot create more than one world and one event manager per program instance!");
    }
}
