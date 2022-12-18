use crate::{Assets, persistent};
use std::path::PathBuf;
use world::{user, System, World};

// Initialize a load and add it to the world
fn init(world: &mut World, user: Option<PathBuf>) {
    // Create a new asset loader / cacher
    let loader = Assets::new(user);

    // Load the default shaders
    persistent!(loader, "engine/shaders/basic.vert");
    /*
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
    persistent!(
        loader,
        "engine/shaders/scene/clustered/clustered.func.glsl"
    );
    persistent!(
        loader,
        "engine/shaders/scene/clustered/clustered.cmpt.glsl"
    );
    persistent!(loader, "engine/shaders/hdri/panorama.frag.glsl");
    persistent!(loader, "engine/shaders/hdri/diffuse.frag.glsl");
    persistent!(loader, "engine/shaders/hdri/specular.frag.glsl");
    persistent!(loader, "engine/shaders/math/sequences.func.glsl");
    persistent!(loader, "engine/shaders/math/conversions.func.glsl");
    persistent!(loader, "engine/shaders/math/kernels.func.glsl");

    // Load the default meshes
    persistent!(loader, "engine/meshes/cube.obj");
    persistent!(loader, "engine/meshes/sphere.obj");

    // Load the default texutres
    persistent!(loader, "engine/textures/integration.png");
    */

    // Insert the loader
    world.insert(loader);
}

// This system will add the asset loader resource into the world and automatically pre-load the default assets as well
// This system will also insert the GlobalPaths resource into the world
pub fn system(system: &mut System, path: Option<PathBuf>) {
    system
        .insert_init(move |world: &mut World| init(world, path))
        .before(user);
}
