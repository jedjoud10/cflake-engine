use crate::{asset, Assets};

use world::{user, System, World};

// Initialize a load and add it to the world
fn init(world: &mut World) {
    let loader = Assets::new();

    macro_rules! internal {
        ($assets:expr, $file:expr) => {
            asset!($assets, $file, "/src/assets/");
        };
    }

    // Load the default common shaders
    internal!(loader, "engine/shaders/common/camera.glsl");
    internal!(loader, "engine/shaders/common/scene.glsl");
    internal!(loader, "engine/shaders/common/timing.glsl");
    internal!(loader, "engine/shaders/common/shadow.glsl");
    internal!(loader, "engine/shaders/common/window.glsl");
    internal!(loader, "engine/shaders/common/sky.glsl");

    // Load the default math shaders
    internal!(loader, "engine/shaders/math/models.glsl");
    internal!(loader, "engine/shaders/math/conversions.glsl");
    internal!(loader, "engine/shaders/math/dither.glsl");
    internal!(loader, "engine/shaders/math/triplanar.glsl");
    internal!(loader, "engine/shaders/math/packer.glsl");

    // Load the default SDF shaders
    internal!(loader, "engine/shaders/sdf/common.glsl");
    internal!(loader, "engine/shaders/sdf/operations.glsl");

    // Load the default rendering shaders
    internal!(loader, "engine/shaders/scene/pbr/pbr.vert");
    internal!(loader, "engine/shaders/scene/pbr/pbr.frag");
    internal!(loader, "engine/shaders/scene/shadow/shadow.frag");
    internal!(loader, "engine/shaders/scene/shadow/shadow.vert");
    internal!(loader, "engine/shaders/scene/shadow/terrain.vert");
    internal!(loader, "engine/shaders/scene/sky/sky.frag");
    internal!(loader, "engine/shaders/scene/sky/sky.vert");
    internal!(loader, "engine/shaders/scene/terrain/terrain.vert");
    internal!(loader, "engine/shaders/scene/terrain/terrain.frag");
    internal!(loader, "engine/shaders/scene/wireframe/wireframe.vert");
    internal!(loader, "engine/shaders/scene/wireframe/wireframe.frag");

    // Load the default post-rendering shaders
    internal!(loader, "engine/shaders/post/display.frag");
    internal!(loader, "engine/shaders/post/display.vert");
    internal!(loader, "engine/shaders/post/gui.vert");
    internal!(loader, "engine/shaders/post/gui.frag");

    // Load the default noise shaders
    internal!(loader, "engine/shaders/noises/cellular2D.glsl");
    internal!(loader, "engine/shaders/noises/cellular2x2.glsl");
    internal!(loader, "engine/shaders/noises/cellular2x2x2.glsl");
    internal!(loader, "engine/shaders/noises/cellular3D.glsl");
    internal!(loader, "engine/shaders/noises/common.glsl");
    internal!(loader, "engine/shaders/noises/noise2D.glsl");
    internal!(loader, "engine/shaders/noises/noise3D.glsl");
    internal!(loader, "engine/shaders/noises/noise3Dgrad.glsl");
    internal!(loader, "engine/shaders/noises/noise4D.glsl");
    internal!(loader, "engine/shaders/noises/fbm.glsl");
    internal!(loader, "engine/shaders/noises/gnoise.glsl");
    internal!(loader, "engine/shaders/noises/erosion2D.glsl");

    // Load the default generation terrain shaders
    internal!(loader, "engine/shaders/terrain/default/voxel.glsl");
    internal!(loader, "engine/shaders/terrain/default/shading.glsl");
    internal!(loader, "engine/shaders/terrain/default/detail.glsl");
    
    // Load the default internally used terrain shaders
    internal!(loader, "engine/shaders/terrain/voxels.comp");
    internal!(loader, "engine/shaders/terrain/vertices.comp");
    internal!(loader, "engine/shaders/terrain/quads.comp");
    internal!(loader, "engine/shaders/terrain/copy.comp");
    internal!(loader, "engine/shaders/terrain/find.comp");

    // Load the default textures
    internal!(loader, "engine/textures/scene/bumps.jpg");

    // Load the default meshes
    internal!(loader, "engine/meshes/cube.obj");
    internal!(loader, "engine/meshes/sphere.obj");
    internal!(loader, "engine/meshes/icosphere.obj");
    internal!(loader, "engine/meshes/plane.obj");

    // Insert the loader
    world.insert(loader);
}

// This system will add the asset loader resource into the world and automatically pre-load the default assets as well
// This system will also insert the GlobalPaths resource into the world
pub fn system(system: &mut System) {
    system.insert_init(init).before(user);
}
