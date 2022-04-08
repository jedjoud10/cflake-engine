use components::*;
use systems::*;
use world::{
    assets::persistent,
    ecs::{registry, EventExecutionOrder},
    World,
};
// Default components
pub mod components;
// Default globals
pub mod globals;
// Default systems
pub mod systems;

// Pre-load the default assets
pub fn preload_default_assets() {
    // Pre load the assets
    println!("Pre-loading default assets...");
    // Rendering
    persistent!("./assets/defaults/shaders/rendering/empty.frsh.glsl");
    persistent!("./assets/defaults/shaders/rendering/project.vrsh.glsl");
    persistent!("./assets/defaults/shaders/rendering/lighting_pass.frsh.glsl");
    persistent!("./assets/defaults/shaders/rendering/default.vrsh.glsl");
    persistent!("./assets/defaults/shaders/rendering/default.frsh.glsl");
    persistent!("./assets/defaults/shaders/rendering/missing.vrsh.glsl");
    persistent!("./assets/defaults/shaders/rendering/missing.frsh.glsl");
    persistent!("./assets/defaults/shaders/rendering/shadows.func.glsl");
    persistent!("./assets/defaults/shaders/rendering/uv_passthrough.vrsh.glsl");
    persistent!("./assets/defaults/shaders/rendering/lighting.func.glsl");
    persistent!("./assets/defaults/shaders/rendering/postprocessing.func.glsl");
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
// Load default systems
pub fn load_default_systems(world: &mut World) {
    // Register the main components
    registry::register::<Transform>();
    registry::register::<Camera>();
    registry::register::<Renderer>();
    registry::register::<Light>();
    registry::register::<RigidBody>();
    registry::register::<Collider>();

    // Engine defaults
    EventExecutionOrder::set(i32::MIN);
    networking_system::system(world);
    camera_system::system(world);
    window_system::system(world);
    audio_system::system(world);

    // User defined systems should start at execution order 0
    EventExecutionOrder::set(0);
    // Terrain
    /*
    terrain_system::chunk_system::system(world);
    terrain_system::voxel_system::system(world);
    terrain_system::mesher_system::system(world);
    terrain_system::editing_system::system(world);
    */
    physics_system::rigidbody_system::system(world);
    physics_system::simulation_system::system(world);

    /*




    // We gotta add the default globals
    */
    debugging_system::system(world);

    EventExecutionOrder::set(i32::MAX - 10);
    rendering_system::system(world);
    gui_system::system(world);
    screenshot_system::system(world);

    world.globals.insert(crate::globals::GlobalWorldData::default()).unwrap();
    world.globals.insert(crate::globals::NetworkManager::default()).unwrap();
    world.globals.insert(crate::globals::Physics::default()).unwrap();
}
/*
// Load the debugging systems
pub fn load_debugging_systems(world: &mut World) {
    SystemExecutionOrder::set(0);
    debugging_system::system(world);
    networking_system::debugging_system::system(world);
    flycam_system::system(world);
}
*/
