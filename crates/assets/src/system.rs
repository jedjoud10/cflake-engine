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

    // Commong GLSL files
    internal!(loader, "engine/shaders/common/conversions.glsl");
    internal!(loader, "engine/shaders/common/dither.glsl");
    internal!(loader, "engine/shaders/common/packer.glsl");
    internal!(loader, "engine/shaders/common/quad.vert");
    internal!(loader, "engine/shaders/common/camera.glsl");
    internal!(loader, "engine/shaders/common/empty.frag");

    // Basic SDF operations taken from Inigo Quilez's website
    internal!(loader, "engine/shaders/sdf/common.glsl");
    internal!(loader, "engine/shaders/sdf/operations.glsl");

    // Load the physically based rendering shader
    internal!(loader, "engine/shaders/scene/pbr/pbr.vert");
    internal!(loader, "engine/shaders/scene/pbr/pbr.frag");

    // Load the custom terrain shader
    internal!(loader, "engine/shaders/scene/terrain/terrain.vert");
    internal!(loader, "engine/shaders/scene/terrain/terrain.frag");

    // Load the default shadow shaders
    internal!(loader, "engine/shaders/scene/terrain/shadow.vert");
    internal!(loader, "engine/shaders/scene/pbr/shadow.vert");

    // Load the default environment compute shaders
    internal!(loader, "engine/shaders/scene/environment/sky.glsl");
    internal!(loader, "engine/shaders/scene/environment/environment.comp");
    internal!(loader, "engine/shaders/scene/environment/diffuse.comp");

    // Load the default post-rendering shaders
    internal!(loader, "engine/shaders/post/lighting.frag");

    // Load the eGUI GUI shaders
    internal!(loader, "engine/shaders/gui/gui.vert");
    internal!(loader, "engine/shaders/gui/gui.frag");

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
    internal!(loader, "engine/shaders/terrain/voxel.glsl");

    // Load the default internally used terrain shaders
    internal!(loader, "engine/shaders/terrain/cull.comp");
    internal!(loader, "engine/shaders/terrain/voxels.comp");
    internal!(loader, "engine/shaders/terrain/vertices.comp");
    internal!(loader, "engine/shaders/terrain/quads.comp");
    internal!(loader, "engine/shaders/terrain/copy.comp");
    internal!(loader, "engine/shaders/terrain/find.comp");
    internal!(loader, "engine/shaders/terrain/utils.glsl");

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
#[doc(hidden)]
pub fn system(system: &mut System) {
    system.insert_init(init).before(user);
}
