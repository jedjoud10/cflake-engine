use systems::*;
use world::{assets::persistent, World};
// Default components
pub mod components;
// Default globals
pub mod globals;
// Default systems
pub mod systems;
pub use world::*;

// Pre-load the default assets
pub fn preload_default_assets() {
    // Pre load the assets
    println!("Pre-loading default assets...");
    // Rendering
    persistent!("./assets/defaults/shaders/rendering/passthrough.vrsh.glsl");
    persistent!("./assets/defaults/shaders/rendering/passthrough.frsh.glsl");
    persistent!("./assets/defaults/shaders/rendering/lighting_pass.frsh.glsl");
    persistent!("./assets/defaults/shaders/rendering/default.vrsh.glsl");
    persistent!("./assets/defaults/shaders/rendering/default.frsh.glsl");
    persistent!("./assets/defaults/shaders/rendering/shadow.vrsh.glsl");
    persistent!("./assets/defaults/shaders/rendering/shadow.frsh.glsl");
    persistent!("./assets/defaults/shaders/rendering/shadow_calculations.func.glsl");
    persistent!("./assets/defaults/shaders/rendering/screen_space_reflections.func.glsl");
    persistent!("./assets/defaults/shaders/rendering/lighting.func.glsl");
    persistent!("./assets/defaults/shaders/rendering/sky.func.glsl");
    persistent!("./assets/defaults/shaders/rendering/sun.func.glsl");
    // Meshes
    persistent!("./assets/defaults/meshes/cube.obj");
    persistent!("./assets/defaults/meshes/sphere.obj");
    // GUI
    persistent!("./assets/defaults/shaders/gui/frag.frsh.glsl");
    persistent!("./assets/defaults/shaders/gui/vert.vrsh.glsl");
    // Others
    persistent!("./assets/defaults/shaders/others/hashes.func.glsl");
    persistent!("./assets/defaults/shaders/others/triplanar.func.glsl");
    persistent!("./assets/defaults/shaders/others/sdf.func.glsl");
    // Default impls
    persistent!("./assets/defaults/shaders/others/default_impls/general.func.glsl");
    persistent!("./assets/defaults/shaders/others/default_impls/renderer.func.glsl");
    // Noise
    persistent!("./assets/defaults/shaders/noises/simplex.func.glsl");
    persistent!("./assets/defaults/shaders/noises/voronoi.func.glsl");
    // Voxel terrain
    persistent!("./assets/defaults/shaders/voxel_terrain/base.cmpt.glsl");
    persistent!("./assets/defaults/shaders/voxel_terrain/second.cmpt.glsl");
    persistent!("./assets/defaults/shaders/voxel_terrain/voxel.func.glsl");
    persistent!("./assets/defaults/shaders/voxel_terrain/shared.func.glsl");
    persistent!("./assets/defaults/shaders/voxel_terrain/edits.func.glsl");
    persistent!("./assets/defaults/shaders/voxel_terrain/terrain.frsh.glsl");
    persistent!("./assets/defaults/shaders/voxel_terrain/terrain.vrsh.glsl");
    // Textures
    persistent!("./assets/defaults/textures/missing.png");
    persistent!("./assets/defaults/textures/sky_gradient.png");

    println!("Finished pre-loading default assets!");
}
// Start the default systems that will be executed before the user systems
pub fn start_before_user_sytems(world: &mut World) {
    // Engine defaults
    camera_system::system(world);
    debugging_system::system(world);
    window_system::system(world);
    audio_system::system(world);

    // We gotta add the default globals
    world.globals.add(crate::globals::GlobalWorldData::default()).unwrap();
    world.globals.add(crate::globals::Physics::default()).unwrap();
}

// Start the defaults systems that will be executed after the user systems
pub fn start_after_user_systems(world: &mut World) {
    physics_system::rigidbody_system::system(world);
    physics_system::simulation_system::system(world);
    light_system::system(world);
    rendering_system::system(world);
    gui_system::system(world);
    // Terrain
    terrain_system::chunk_system::system(world);
    terrain_system::voxel_system::system(world);
    terrain_system::mesher_system::system(world);
    terrain_system::editing_system::system(world);
}
