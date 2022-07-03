use crate::{persistent, Assets};
use std::path::PathBuf;
use world::{Events, Init, Stage, World};

// This system will add the asset loader resource into the world and automatically pre-load the default assets as well
// This system will also insert the GlobalPaths resource into the world
pub fn system(events: &mut Events, user: Option<PathBuf>) {
    // Insert the asset loader and load the default assets
    events
        .registry::<Init>()
        .insert_with(
            move |world: &mut World| {
                // Create a new asset loader / cacher
                let mut loader = Assets::new(user.clone());

                // Load the default shaders
                persistent!(loader, "engine/shaders/pbr.vrsh.glsl");
                persistent!(loader, "engine/shaders/pbr.frsh.glsl");

                // Insert the loader
                world.insert(loader);
            },
            Stage::new("asset loader insert").before("user"),
        )
        .unwrap();
}
