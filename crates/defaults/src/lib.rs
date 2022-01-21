// Default components
pub mod components;
// Default systems
pub mod template_system;
pub mod systems;


use main::core::{WriteContext, TaskSenderContext};
use main::assets::preload_asset;
use main::ecs;
use systems::camera_system;
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
    // Default impls
    preload_asset!(".\\resources\\defaults\\shaders\\others\\default_impls\\general.func.glsl");
    preload_asset!(".\\resources\\defaults\\shaders\\others\\default_impls\\renderer.func.glsl");
    preload_asset!(".\\resources\\defaults\\shaders\\others\\default_impls\\renderer_life_fade.func.glsl");
    preload_asset!(".\\resources\\defaults\\shaders\\others\\default_impls\\renderer_main_start.func.glsl");
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
pub fn preload_system(mut write: WriteContext, task_sender: TaskSenderContext) {
    template_system::system(&mut write);
    camera_system::system(&mut write);
    /*

    // We want to read the current time from the world
    let read_context = context.read();
    let time = read_context.time.elapsed;
    //dbg!(time);
    */
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
