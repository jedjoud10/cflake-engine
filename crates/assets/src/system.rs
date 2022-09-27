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
                let loader = Assets::new(user.clone());

                // Load the default shaders
                persistent!(loader, "engine/shaders/models.func.glsl");
                persistent!(loader, "engine/shaders/pbr.vrtx.glsl");
                persistent!(loader, "engine/shaders/pbr.frag.glsl");
                persistent!(loader, "engine/shaders/gui.vrtx.glsl");
                persistent!(loader, "engine/shaders/gui.frag.glsl");
                persistent!(loader, "engine/shaders/sky.frag.glsl");
                persistent!(loader, "engine/shaders/passthrough.vrtx.glsl");
                persistent!(loader, "engine/shaders/compositor.frag.glsl");
                persistent!(loader, "engine/shaders/projection.vrtx.glsl");
                persistent!(loader, "engine/shaders/depth.frag.glsl");
                persistent!(loader, "engine/shaders/shadow.func.glsl");
                persistent!(loader, "engine/shaders/clustered.func.glsl");
                persistent!(loader, "engine/shaders/clustered.cmpt.glsl");
                persistent!(loader, "engine/shaders/panorama.frag.glsl");

                // Load the default meshes
                persistent!(loader, "engine/meshes/cube.obj");
                persistent!(loader, "engine/meshes/sphere.obj");

                // Load the default textures
                persistent!(loader, "engine/textures/bumps.png");
                persistent!(loader, "engine/textures/missing.png");
                persistent!(loader, "engine/textures/sky_gradient.png");

                // Insert the loader
                world.insert(loader);
            },
            Stage::new("asset loader insert").before("user"),
        )
        .unwrap();
}
