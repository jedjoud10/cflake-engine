use main::{assets::cache_persistent, core::World};
use systems::*;
// Default components
pub mod components;
// Default globals
pub mod globals;
// Default systems
pub mod systems;
pub mod template_system;

// Pre-load the default assets
pub fn preload_default_assets() {
    // Pre load the assets
    log::info!("Pre-loading default assets...");
    // Rendering
    cache_persistent!("./assets/defaults/shaders/rendering/passthrough.vrsh.glsl");
    cache_persistent!("./assets/defaults/shaders/rendering/lighting_pass.frsh.glsl");
    cache_persistent!("./assets/defaults/shaders/rendering/postprocessing_pass.frsh.glsl");
    cache_persistent!("./assets/defaults/shaders/rendering/default.vrsh.glsl");
    cache_persistent!("./assets/defaults/shaders/rendering/default.frsh.glsl");
    cache_persistent!("./assets/defaults/shaders/rendering/shadow.vrsh.glsl");
    cache_persistent!("./assets/defaults/shaders/rendering/shadow.frsh.glsl");
    cache_persistent!("./assets/defaults/shaders/rendering/shadow_calculations.func.glsl");
    cache_persistent!("./assets/defaults/shaders/rendering/screen_space_reflections.func.glsl");
    cache_persistent!("./assets/defaults/shaders/rendering/lighting.func.glsl");
    cache_persistent!("./assets/defaults/shaders/rendering/sky.func.glsl");
    cache_persistent!("./assets/defaults/shaders/rendering/sun.func.glsl");
    // GUI
    cache_persistent!("./assets/defaults/shaders/gui/frag.frsh.glsl");
    cache_persistent!("./assets/defaults/shaders/gui/vert.vrsh.glsl");
    // Others
    cache_persistent!("./assets/defaults/shaders/others/hashes.func.glsl");
    cache_persistent!("./assets/defaults/shaders/others/triplanar.func.glsl");
    cache_persistent!("./assets/defaults/shaders/others/sdf.func.glsl");
    // Default impls
    cache_persistent!("./assets/defaults/shaders/others/default_impls/general.func.glsl");
    cache_persistent!("./assets/defaults/shaders/others/default_impls/renderer.func.glsl");
    // Models
    cache_persistent!("./assets/defaults/models/sphere.mdl3d");
    cache_persistent!("./assets/defaults/models/quad.mdl3d");
    cache_persistent!("./assets/defaults/models/cube.mdl3d");
    // Noise
    cache_persistent!("./assets/defaults/shaders/noises/simplex.func.glsl");
    cache_persistent!("./assets/defaults/shaders/noises/voronoi.func.glsl");
    // Voxel terrain
    cache_persistent!("./assets/defaults/shaders/voxel_terrain/base.cmpt.glsl");
    cache_persistent!("./assets/defaults/shaders/voxel_terrain/second.cmpt.glsl");
    cache_persistent!("./assets/defaults/shaders/voxel_terrain/voxel.func.glsl");
    cache_persistent!("./assets/defaults/shaders/voxel_terrain/shared.func.glsl");
    cache_persistent!("./assets/defaults/shaders/voxel_terrain/edits.func.glsl");
    cache_persistent!("./assets/defaults/shaders/voxel_terrain/terrain.frsh.glsl");
    cache_persistent!("./assets/defaults/shaders/voxel_terrain/terrain.vrsh.glsl");
    // Textures
    cache_persistent!("./assets/defaults/textures/missing_texture.png");
    cache_persistent!("./assets/defaults/textures/sky_gradient.png");

    log::info!("Finished pre-loading default assets!");
}
// Pre-load the default systems
pub fn preload_system(world: &mut World) {
    //template_system::system(world);
    camera_system::system(world);
    //physics_system::system(world);
    rendering_system::system(world);
    debugging_system::system(world);
    window_system::system(world);
    gui_system::system(world);
    test_system::system(world);
    audio_system::system(world);
    // Terrain
    terrain_system::chunk_system::system(world);
    terrain_system::voxel_system::system(world);
    terrain_system::mesher_system::system(world);
    terrain_system::mesh_update_system::system(world);
    terrain_system::editing_system::system(world);

    // We gotta add the globa
    world
        .globals
        .add_global(crate::globals::GlobalWorldData::default())
        .unwrap();
}
