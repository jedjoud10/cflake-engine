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

    // Load the default common shaders
    persistent!(loader, "engine/shaders/common/camera.glsl");
    persistent!(loader, "engine/shaders/common/scene.glsl");
    persistent!(loader, "engine/shaders/common/timing.glsl");
    persistent!(loader, "engine/shaders/common/shadow.glsl");
    persistent!(loader, "engine/shaders/common/window.glsl");
    persistent!(loader, "engine/shaders/common/sky.glsl");
    persistent!(loader, "engine/shaders/common/extensions.glsl");

    // Load the default math shaders
    persistent!(loader, "engine/shaders/math/models.glsl");
    persistent!(loader, "engine/shaders/math/conversions.glsl");

    // Load the default rendering shaders
    persistent!(loader, "engine/shaders/scene/basic/basic.frag");
    persistent!(loader, "engine/shaders/scene/pbr/pbr.frag");
    persistent!(loader, "engine/shaders/scene/basic/basic.vert");
    persistent!(loader, "engine/shaders/scene/shadow/shadow.frag");
    persistent!(loader, "engine/shaders/scene/shadow/shadow.vert");
    persistent!(loader, "engine/shaders/scene/sky/sky.frag");
    persistent!(loader, "engine/shaders/scene/sky/sky.vert");
    persistent!(loader, "engine/shaders/scene/terrain/terrain.vert");
    persistent!(loader, "engine/shaders/scene/terrain/terrain.frag");

    // Load the default post-rendering shaders
    persistent!(loader, "engine/shaders/post/display.frag");
    persistent!(loader, "engine/shaders/post/display.vert");
    persistent!(loader, "engine/shaders/post/gui.vert");
    persistent!(loader, "engine/shaders/post/gui.frag");

    // Load the default noise shaders
    persistent!(loader, "engine/shaders/noises/cellular2D.glsl");
    persistent!(loader, "engine/shaders/noises/cellular2x2.glsl");
    persistent!(loader, "engine/shaders/noises/cellular2x2x2.glsl");
    persistent!(loader, "engine/shaders/noises/cellular3D.glsl");
    persistent!(loader, "engine/shaders/noises/common.glsl");
    persistent!(loader, "engine/shaders/noises/noise2D.glsl");
    persistent!(loader, "engine/shaders/noises/noise3D.glsl");
    persistent!(loader, "engine/shaders/noises/noise3Dgrad.glsl");
    persistent!(loader, "engine/shaders/noises/noise4D.glsl");
    persistent!(loader, "engine/shaders/noises/fbm.glsl");
    persistent!(loader, "engine/shaders/noises/gnoise.glsl");

    // Load the default terrain shaders
    persistent!(loader, "engine/shaders/terrain/voxel.comp");
    persistent!(loader, "engine/shaders/terrain/vertices.comp");
    persistent!(loader, "engine/shaders/terrain/quads.comp");

    // Load the default textures
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
