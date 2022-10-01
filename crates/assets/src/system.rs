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
                persistent!(loader, "engine/shaders/scene/pbr/models.func.glsl");
                persistent!(loader, "engine/shaders/scene/pbr/pbr.vrtx.glsl");
                persistent!(loader, "engine/shaders/scene/pbr/pbr.frag.glsl");
                persistent!(loader, "engine/shaders/gui.vrtx.glsl");
                persistent!(loader, "engine/shaders/gui.frag.glsl");
                persistent!(loader, "engine/shaders/sky.frag.glsl");
                persistent!(loader, "engine/shaders/passthrough.vrtx.glsl");
                persistent!(loader, "engine/shaders/compositor.frag.glsl");
                persistent!(loader, "engine/shaders/projection.vrtx.glsl");
                persistent!(loader, "engine/shaders/depth.frag.glsl");
                persistent!(loader, "engine/shaders/scene/shadow.func.glsl");
                persistent!(loader, "engine/shaders/scene/clustered/clustered.func.glsl");
                persistent!(loader, "engine/shaders/scene/clustered/clustered.cmpt.glsl");
                persistent!(loader, "engine/shaders/hdri/panorama.frag.glsl");
                persistent!(loader, "engine/shaders/hdri/diffuse.frag.glsl");
                persistent!(loader, "engine/shaders/hdri/specular.frag.glsl");
                persistent!(loader, "engine/shaders/math/sequences.func.glsl");
                persistent!(loader, "engine/shaders/math/conversions.func.glsl");

                // Load the default meshes
                persistent!(loader, "engine/meshes/cube.obj");
                persistent!(loader, "engine/meshes/sphere.obj");

                // Load the default texutres
                persistent!(loader, "engine/textures/integration.png");

                // Insert the loader
                world.insert(loader);
            },
            Stage::new("asset loader insert").before("user"),
        )
        .unwrap();
}
