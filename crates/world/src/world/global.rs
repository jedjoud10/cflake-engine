use crate::{Events, World};
use std::sync::atomic::{AtomicBool, Ordering};

// The setup function can only be called once
static INITIALIZED: AtomicBool = AtomicBool::new(false);

// This global function will be used to initialize the Events and the World
// This will be called by the main glutin handler, but it can only be called once
pub fn setup() -> (World, Events) {
    if !INITIALIZED.fetch_or(true, Ordering::Relaxed) {
        (
            // Create a single instance of the world
            World(Default::default()),
            // Create a single instance of the events
            Events {
                window: Default::default(),
                device: Default::default(),
                init: Default::default(),
                update: Default::default(),
                shutdown: Default::default(),
            },
        )
    } else {
        // We've already created the world and event manager, so we must panic
        panic!("Cannot create more than one world and one event manager per program instance!");
    }
}
