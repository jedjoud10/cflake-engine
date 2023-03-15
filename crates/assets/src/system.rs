use crate::{persistent, Assets};
use std::path::PathBuf;
use world::{user, System, World};

// Simple resource that is temporarily added to world to pass user assets path
pub struct AssetsSettings(pub Option<PathBuf>);

// Initialize a load and add it to the world
fn init(world: &mut World) {
    // Create a new asset loader / cacher
    let user = world.get::<AssetsSettings>().unwrap().0.clone();
    let loader = Assets::new(user);

    // Load the default shaders
    persistent!(loader, "engine/shaders/common/camera.glsl");
    persistent!(loader, "engine/shaders/common/scene.glsl");
    persistent!(loader, "engine/shaders/common/timing.glsl");
    persistent!(loader, "engine/shaders/common/shadow.glsl");
    persistent!(loader, "engine/shaders/common/window.glsl");
    persistent!(loader, "engine/shaders/common/sky.glsl");
    persistent!(loader, "engine/shaders/common/extensions.glsl");
    persistent!(loader, "engine/shaders/math/models.glsl");
    persistent!(loader, "engine/shaders/math/conversions.glsl");

    persistent!(loader, "engine/shaders/scene/basic/basic.frag");
    persistent!(loader, "engine/shaders/scene/basic/basic.vert");
    persistent!(loader, "engine/shaders/scene/shadow/shadow.frag");
    persistent!(loader, "engine/shaders/scene/shadow/shadow.vert");
    persistent!(loader, "engine/shaders/scene/sky/sky.frag");
    persistent!(loader, "engine/shaders/scene/sky/sky.vert");
    persistent!(loader, "engine/shaders/post/display.frag");
    persistent!(loader, "engine/shaders/post/display.vert");
    persistent!(loader, "engine/shaders/post/gui.vert");
    persistent!(loader, "engine/shaders/post/gui.frag");

    // Load the default textures
    persistent!(loader, "engine/textures/scene/sky.jpg");
    persistent!(loader, "engine/textures/scene/bumps.jpg");

    // Load the default meshes
    persistent!(loader, "engine/meshes/cube.obj");
    persistent!(loader, "engine/meshes/sphere.obj");
    persistent!(loader, "engine/meshes/icosphere.obj");
    persistent!(loader, "engine/meshes/plane.obj");

    // Insert the loader
    world.insert(loader);
}

// This system will add the asset loader resource into the world and automatically pre-load the default assets as well
// This system will also insert the GlobalPaths resource into the world
pub fn system(system: &mut System) {
    system.insert_init(init).before(user);
}
