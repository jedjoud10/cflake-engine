#![feature(int_roundings)]
use main::assets::preload_asset;
use main::core::WriteContext;
use systems::{camera_system, debugging_system, physics_system, rendering_system, window_system, terrain};
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
    preload_asset!(".\\resources\\defaults\\shaders\\others\\dithering.func.glsl");
    preload_asset!(".\\resources\\defaults\\shaders\\others\\template_compute.cmpt.glsl");
    // Default impls
    preload_asset!(".\\resources\\defaults\\shaders\\others\\default_impls\\general.func.glsl");
    preload_asset!(".\\resources\\defaults\\shaders\\others\\default_impls\\renderer.func.glsl");
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
    preload_asset!(".\\resources\\defaults\\shaders\\voxel_terrain\\base.cmpt.glsl");
    preload_asset!(".\\resources\\defaults\\shaders\\voxel_terrain\\second.cmpt.glsl");
    preload_asset!(".\\resources\\defaults\\shaders\\voxel_terrain\\voxel.func.glsl");
    preload_asset!(".\\resources\\defaults\\shaders\\voxel_terrain\\shared.func.glsl");
    preload_asset!(".\\resources\\defaults\\shaders\\voxel_terrain\\terrain_shader.func.glsl");
    preload_asset!(".\\resources\\defaults\\shaders\\voxel_terrain\\terrain.frsh.glsl");
    // Textures
    preload_asset!(".\\resources\\defaults\\textures\\missing_texture.png");
    preload_asset!(".\\resources\\defaults\\textures\\sky_gradient.png");

    println!("Finished pre-loading default assets!");
}
// Pre-load the default systems
pub fn preload_system(mut write: WriteContext) {
    template_system::system(&mut write);
    camera_system::system(&mut write);
    physics_system::system(&mut write);
    rendering_system::system(&mut write);
    debugging_system::system(&mut write);
    window_system::system(&mut write);
    //test_system::system(&mut write);
    // Terrain
    terrain::chunk_system::system(&mut write);
    terrain::voxel_system::system(&mut write);
    terrain::mesher_system::system(&mut write);   

    // We gotta add the global world data
    write.ecs.add_global(crate::globals::GlobalWorldData::default()).unwrap();
}
/*
pub fn preload_systems() {
    let mut interpreter = terrain::interpreter::Interpreter::new_pregenerated();
    let (string, csgtree) = interpreter.finalize().unwrap();
    let terrain_shader = rendering::pipec::shader(
        Shader::default()
            .load_shader(vec![
                "defaults\\shaders\\rendering\\default.vrsh.glsl",
                "defaults\\shaders\\voxel_terrain\\terrain.frsh.glsl",
            ])
            .unwrap(),
    );

    let mut material = Material::new("Terrain material").set_shader(terrain_shader);
    material.uniforms.set_f32("normals_strength", 2.0);
    material.uniforms.set_vec2f32("uv_scale", veclib::Vector2::ONE * 0.7);

    let texture =
        rendering::pipec::texture(Texture::create_texturearray(vec!["defaults\\textures\\rock_diffuse.png", "defaults\\textures\\missing_texture.png"], 256, 256).enable_mipmaps());
    let texture2 =
        rendering::pipec::texture(Texture::create_texturearray(vec!["defaults\\textures\\rock_normal.png", "defaults\\textures\\missing_texture.png"], 256, 256).enable_mipmaps());
    material.uniforms.set_t2da("diffuse_textures", &texture, 0);
    material.uniforms.set_t2da("normals_textures", &texture2, 1);
    material.uniforms.set_i32("max_depth", 8);

    default_system::system();
    systems::terrain::mesher_system::system(rendering::pipec::material(material));
    systems::terrain::chunk_system::system(8, csgtree);
    systems::terrain::voxel_generation_system::system(string);
    systems::rendering_system::system();
    systems::camera_system::system();
}
*/
