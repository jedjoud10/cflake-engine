#![feature(int_roundings)]
use main::{assets::preload_asset, core::World};
use systems::{camera_system, debugging_system, rendering_system, terrain, ui_system, window_system};
// Default components
pub mod components;
// Default globals
pub mod globals;
// Default systems
pub mod systems;
pub mod template_system;

// Pre-load the default assets
pub fn preload_default_assets() {
    // Pre load the resources
    println!("Pre-loading default assets...");
    // Rendering
    preload_asset!(".\\resources\\defaults\\shaders\\rendering\\passthrough.vrsh.glsl");
    preload_asset!(".\\resources\\defaults\\shaders\\rendering\\lighting_pass.frsh.glsl");
    preload_asset!(".\\resources\\defaults\\shaders\\rendering\\default.vrsh.glsl");
    preload_asset!(".\\resources\\defaults\\shaders\\rendering\\default.frsh.glsl");
    preload_asset!(".\\resources\\defaults\\shaders\\rendering\\shadow.vrsh.glsl");
    preload_asset!(".\\resources\\defaults\\shaders\\rendering\\shadow.frsh.glsl");
    preload_asset!(".\\resources\\defaults\\shaders\\rendering\\shadow_calculations.func.glsl");
    preload_asset!(".\\resources\\defaults\\shaders\\rendering\\screen_space_reflections.func.glsl");
    preload_asset!(".\\resources\\defaults\\shaders\\rendering\\lighting.func.glsl");
    preload_asset!(".\\resources\\defaults\\shaders\\rendering\\sky.func.glsl");
    preload_asset!(".\\resources\\defaults\\shaders\\rendering\\sun.func.glsl");
    // Others
    preload_asset!(".\\resources\\defaults\\shaders\\others\\hashes.func.glsl");
    preload_asset!(".\\resources\\defaults\\shaders\\others\\triplanar.func.glsl");
    preload_asset!(".\\resources\\defaults\\shaders\\others\\sdf.func.glsl");
    preload_asset!(".\\resources\\defaults\\shaders\\others\\dithering.func.glsl");
    preload_asset!(".\\resources\\defaults\\shaders\\others\\template_compute.cmpt.glsl");
    // Default impls
    preload_asset!(".\\resources\\defaults\\shaders\\others\\default_impls\\general.func.glsl");
    preload_asset!(".\\resources\\defaults\\shaders\\others\\default_impls\\renderer.func.glsl");
    // Models
    preload_asset!(".\\resources\\defaults\\models\\sphere.mdl3d");
    preload_asset!(".\\resources\\defaults\\models\\quad.mdl3d");
    preload_asset!(".\\resources\\defaults\\models\\cube.mdl3d");
    // Noise
    preload_asset!(".\\resources\\defaults\\shaders\\noises\\simplex.func.glsl");
    preload_asset!(".\\resources\\defaults\\shaders\\noises\\voronoi.func.glsl");
    // Voxel terrain
    preload_asset!(".\\resources\\defaults\\shaders\\voxel_terrain\\base.cmpt.glsl");
    preload_asset!(".\\resources\\defaults\\shaders\\voxel_terrain\\second.cmpt.glsl");
    preload_asset!(".\\resources\\defaults\\shaders\\voxel_terrain\\voxel.func.glsl");
    preload_asset!(".\\resources\\defaults\\shaders\\voxel_terrain\\shared.func.glsl");
    preload_asset!(".\\resources\\defaults\\shaders\\voxel_terrain\\terrain.frsh.glsl");
    preload_asset!(".\\resources\\defaults\\shaders\\voxel_terrain\\terrain.vrsh.glsl");
    // Textures
    preload_asset!(".\\resources\\defaults\\textures\\missing_texture.png");
    preload_asset!(".\\resources\\defaults\\textures\\sky_gradient.png");

    println!("Finished pre-loading default assets!");
}
// Pre-load the default systems
pub fn preload_system(world: &mut World) {
    //template_system::system(world);
    camera_system::system(world);
    //physics_system::system(world);
    rendering_system::system(world);
    debugging_system::system(world);
    window_system::system(world);
    ui_system::system(world);
    //test_system::system(world);
    // Terrain
    terrain::chunk_system::system(world);
    terrain::voxel_system::system(world);
    terrain::mesher_system::system(world);

    // We gotta add the globa
    world.globals.add_global(crate::globals::GlobalWorldData::default()).unwrap();
}
