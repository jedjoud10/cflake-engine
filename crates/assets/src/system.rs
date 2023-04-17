use crate::{asset, Assets, UserAssets};
use std::path::PathBuf;
use world::{user, System, World};

// Simple resource that is temporarily added to world to pass user assets
pub struct AssetsSettings(pub Option<UserAssets>);

// Initialize a load and add it to the world
fn init(world: &mut World) {
    // Create a new asset loader / cacher
    let user = world.remove::<AssetsSettings>().unwrap();
    let loader = Assets::new(user.0);

    // Load the default common shaders
    asset!(loader, "engine/shaders/common/camera.glsl");
    asset!(loader, "engine/shaders/common/scene.glsl");
    asset!(loader, "engine/shaders/common/timing.glsl");
    asset!(loader, "engine/shaders/common/shadow.glsl");
    asset!(loader, "engine/shaders/common/window.glsl");
    asset!(loader, "engine/shaders/common/sky.glsl");

    // Load the default math shaders
    asset!(loader, "engine/shaders/math/models.glsl");
    asset!(loader, "engine/shaders/math/conversions.glsl");
    asset!(loader, "engine/shaders/math/dither.glsl");

    // Load the default SDF shaders
    asset!(loader, "engine/shaders/sdf/common.glsl");
    asset!(loader, "engine/shaders/sdf/operations.glsl");

    // Load the default rendering shaders
    asset!(loader, "engine/shaders/scene/basic/basic.frag");
    asset!(loader, "engine/shaders/scene/pbr/pbr.frag");
    asset!(loader, "engine/shaders/scene/basic/basic.vert");
    asset!(loader, "engine/shaders/scene/shadow/shadow.frag");
    asset!(loader, "engine/shaders/scene/shadow/shadow.vert");
    asset!(loader, "engine/shaders/scene/sky/sky.frag");
    asset!(loader, "engine/shaders/scene/sky/sky.vert");
    asset!(loader, "engine/shaders/scene/terrain/terrain.vert");
    asset!(loader, "engine/shaders/scene/terrain/terrain.frag");

    // Load the default post-rendering shaders
    asset!(loader, "engine/shaders/post/display.frag");
    asset!(loader, "engine/shaders/post/display.vert");
    asset!(loader, "engine/shaders/post/gui.vert");
    asset!(loader, "engine/shaders/post/gui.frag");

    // Load the default noise shaders
    asset!(loader, "engine/shaders/noises/cellular2D.glsl");
    asset!(loader, "engine/shaders/noises/cellular2x2.glsl");
    asset!(loader, "engine/shaders/noises/cellular2x2x2.glsl");
    asset!(loader, "engine/shaders/noises/cellular3D.glsl");
    asset!(loader, "engine/shaders/noises/common.glsl");
    asset!(loader, "engine/shaders/noises/noise2D.glsl");
    asset!(loader, "engine/shaders/noises/noise3D.glsl");
    asset!(loader, "engine/shaders/noises/noise3Dgrad.glsl");
    asset!(loader, "engine/shaders/noises/noise4D.glsl");
    asset!(loader, "engine/shaders/noises/fbm.glsl");
    asset!(loader, "engine/shaders/noises/gnoise.glsl");
    asset!(loader, "engine/shaders/noises/erosion2D.glsl");

    // Load the default terrain shaders
    asset!(loader, "engine/shaders/terrain/voxels.comp");
    asset!(loader, "engine/shaders/terrain/default.glsl");
    asset!(loader, "engine/shaders/terrain/vertices.comp");
    asset!(loader, "engine/shaders/terrain/quads.comp");
    asset!(loader, "engine/shaders/terrain/copy.comp");
    asset!(loader, "engine/shaders/terrain/find.comp");

    // Load the default textures
    asset!(loader, "engine/textures/scene/bumps.jpg");

    // Load the default meshes
    asset!(loader, "engine/meshes/cube.obj");
    asset!(loader, "engine/meshes/sphere.obj");
    asset!(loader, "engine/meshes/icosphere.obj");
    asset!(loader, "engine/meshes/plane.obj");

    // Insert the loader
    world.insert(loader);
}

// This system will add the asset loader resource into the world and automatically pre-load the default assets as well
// This system will also insert the GlobalPaths resource into the world
pub fn system(system: &mut System) {
    system.insert_init(init).before(user);
}
