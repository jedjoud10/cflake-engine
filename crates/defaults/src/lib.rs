// Default components
pub mod components;
// Default systems
pub mod default_system;
pub mod systems;

use assets::preload_asset;
// Pre-load the default assets
pub fn preload_default_assets() {
    // Pre load the resources
    println!("Pre-loading default assets...");
    // Rendering
    preload_asset!(".\\resources\\defaults\\shaders\\rendering\\passthrough.vrsh.glsl");
    preload_asset!(".\\resources\\defaults\\shaders\\rendering\\screen.frsh.glsl");
    preload_asset!(".\\resources\\defaults\\shaders\\rendering\\default.vrsh.glsl");
    preload_asset!(".\\resources\\defaults\\shaders\\rendering\\default.frsh.glsl");
    // Others
    preload_asset!(".\\resources\\defaults\\shaders\\others\\wireframe.frsh.glsl");
    preload_asset!(".\\resources\\defaults\\shaders\\others\\hashes.func.glsl");
    preload_asset!(".\\resources\\defaults\\shaders\\others\\triplanar.func.glsl");
    preload_asset!(".\\resources\\defaults\\shaders\\others\\debug.vrsh.glsl");
    preload_asset!(".\\resources\\defaults\\shaders\\others\\debug.frsh.glsl");
    preload_asset!(".\\resources\\defaults\\shaders\\others\\frame_stats.cmpt.glsl");
    preload_asset!(".\\resources\\defaults\\shaders\\others\\sdf.func.glsl");
    // UI
    preload_asset!(".\\resources\\defaults\\shaders\\ui\\ui_elem.vrsh.glsl");
    preload_asset!(".\\resources\\defaults\\shaders\\ui\\ui_panel.frsh.glsl");
    preload_asset!(".\\resources\\defaults\\shaders\\ui\\ui_font.vrsh.glsl");
    preload_asset!(".\\resources\\defaults\\shaders\\ui\\ui_font.frsh.glsl");
    preload_asset!(".\\resources\\defaults\\fonts\\default_font.font");
    // Models
    preload_asset!(".\\resources\\defaults\\models\\screen_quad.mdl3d");
    preload_asset!(".\\resources\\defaults\\models\\sphere.mdl3d");
    preload_asset!(".\\resources\\defaults\\models\\quad.mdl3d");
    preload_asset!(".\\resources\\defaults\\models\\cube.mdl3d");
    // Noise
    preload_asset!(".\\resources\\defaults\\shaders\\noises\\simplex.func.glsl");
    preload_asset!(".\\resources\\defaults\\shaders\\noises\\voronoi.func.glsl");
    // Voxel terrain
    preload_asset!(".\\resources\\defaults\\shaders\\voxel_terrain\\voxel_main.cmpt.glsl");
    preload_asset!(".\\resources\\defaults\\shaders\\voxel_terrain\\data.func.glsl");
    preload_asset!(".\\resources\\defaults\\shaders\\voxel_terrain\\terrain_shader.func.glsl");
    preload_asset!(".\\resources\\defaults\\shaders\\voxel_terrain\\terrain.frsh.glsl");
    // Textures
    preload_asset!(".\\resources\\defaults\\textures\\missing_texture.png");
    preload_asset!(".\\resources\\defaults\\textures\\sky_gradient.png");
    preload_asset!(".\\resources\\defaults\\textures\\rock_diffuse.png");
    preload_asset!(".\\resources\\defaults\\textures\\rock_normal.png");
    println!("Finished pre-loading default assets!");
}

// Pre-load the default systems
pub fn preload_systems() {
    default_system::system();

    let mut linkings = ecs::ComponentLinkingGroup::new();
    ecs::registry::register_component::<crate::components::Physics>();
    linkings.link_default::<crate::components::Transform>().unwrap();
    linkings.link_default::<crate::components::Physics>().unwrap();
    // Add the entity
    let result = core::global::ecs::entity_add(ecs::Entity::new("Test-Entity"), linkings);
}
