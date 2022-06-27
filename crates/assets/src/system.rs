use std::path::PathBuf;
use world::{World, Events, Init};
use crate::{asset, persistent, Assets};

// This system will add the asset loader resource into the world and automatically pre-load the default assets as well
pub fn system(events: &mut Events, user: Option<PathBuf>) {
    // Insert the asset loader and load the default assets
    events.register::<Init>(move |world: &mut World| {
        // Create a new asset loader / cacher
        let mut loader = Assets::new(user.clone());

        // Load the default shaders
        persistent!(loader, "engine/shaders/pbr.vrsh.glsl");
        persistent!(loader, "engine/shaders/pbr.frsh.glsl");

        // Insert the loader
        world.insert(loader);
    });
}