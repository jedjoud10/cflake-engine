use cflake_engine::{
    assets::assetc,
    defaults::{
        components::{self, Camera, Light, Transform},
        globals::{self, TerrainSettings},
    },
    math::octrees::HeuristicSettings,
    rendering::basics::{
        lights::{
            LightParameters,
            LightType::{self, Directional},
        },
        material::Material,
        shader::{Shader, ShaderInitSettings},
        texture::{Texture2D, TextureFilter, TextureFlags, TextureParams, bundle},
        uniforms::UniformsSet,
    },
    terrain::editing::{Edit, EditParams},
    vek, World,
};

// A game with some test terrain
fn main() {
    cflake_engine::start("cflake-examples/terrain", init)
}
// Init the terrain world
fn init(world: &mut World) {
    cflake_engine::assets::init!("/examples/assets/");
    cflake_engine::assets::asset!("./assets/user/shaders/voxel_terrain/voxel.func.glsl");
    cflake_engine::assets::asset!("./assets/user/shaders/voxel_terrain/terrain.frsh.glsl");
    cflake_engine::assets::asset!("./assets/user/textures/Snow006_2K_Color.jpg");
    cflake_engine::assets::asset!("./assets/user/textures/Snow006_2K_NormalGL.jpg");
    cflake_engine::assets::asset!("./assets/user/textures/rocks_ground_06_diff_2k.jpg");
    cflake_engine::assets::asset!("./assets/user/textures/rocks_ground_06_nor_gl_2k.jpg");
    cflake_engine::assets::asset!("./assets/user/textures/rocks_ground_08_diff_2k.jpg");
    cflake_engine::assets::asset!("./assets/user/textures/rocks_ground_08_nor_gl_2k.jpg");
    cflake_engine::assets::asset!("./assets/user/textures/snow_01_diff_8k.jpg");
    cflake_engine::assets::asset!("./assets/user/textures/snow_01_nor_gl_8k.jpg");
    // Load le assets

    cflake_engine::defaults::systems::flycam_system::system(world);

    // ----Start the world----
    // Create a simple camera entity
    world.ecs.insert(|_, linker| {
        linker.insert(Camera::new(90.0, 2.0, 9000.0)).unwrap();
        linker.insert(Transform::default()).unwrap();
    });

    // Create the directional light source
    world.ecs.insert(|_, linker| {
        let light = Light(LightType::new_directional(0.9, vek::Rgb::one()));
        linker.insert(light).unwrap();
        linker.insert(Transform::rotation_x(-45f32.to_radians())).unwrap();
    });

    // Load a terrain material
    // Load the shader first
    let settings = ShaderInitSettings::default()
        .source("defaults/shaders/voxel_terrain/terrain.vrsh.glsl")
        .source("user/shaders/voxel_terrain/terrain.frsh.glsl");
    let shader = world.pipeline.insert(Shader::new(settings).unwrap());
    // Then the textures
    let diffuse = TextureParams::DIFFUSE_MAP_LOAD;
    let normal = TextureParams::NORMAL_MAP_LOAD;
    let texture_diff_1 = assetc::load_with::<Texture2D>("user/textures/Snow006_2K_Color.jpg", diffuse.clone()).unwrap();
    let texture_norm_1 = assetc::load_with::<Texture2D>("user/textures/Snow006_2K_NormalGL.jpg", normal.clone()).unwrap();
    let texture_diff_2 = assetc::load_with::<Texture2D>("user/textures/rocks_ground_06_diff_2k.jpg", diffuse).unwrap();
    let texture_norm_2 = assetc::load_with::<Texture2D>("user/textures/rocks_ground_06_nor_gl_2k.jpg", normal).unwrap();
    /*
    let texture_diff_1 = assetc::load::<Texture2D>("user/textures/forrest_ground_01_diff_2k.jpg").unwrap();
    let texture_norm_1 = assetc::load::<Texture2D>("user/textures/forrest_ground_01_nor_gl_2k.jpg").unwrap();
    let texture_diff_2 = assetc::load::<Texture2D>("user/textures/rocks_ground_06_diff_2k.jpg").unwrap();
    let texture_norm_2 = assetc::load::<Texture2D>("user/textures/rocks_ground_06_nor_gl_2k.jpg").unwrap();
    let texture_diff_3 = assetc::load::<Texture2D>("user/textures/rocks_ground_08_diff_2k.jpg").unwrap();
    let texture_norm_3 = assetc::load::<Texture2D>("user/textures/rocks_ground_08_nor_gl_2k.jpg").unwrap();
    */
    let diffuse = bundle(&[texture_diff_1, texture_diff_2]).unwrap();
    let normals = bundle(&[texture_norm_1, texture_norm_2]).unwrap();
    let diffuse = world.pipeline.insert(diffuse);
    let normals = world.pipeline.insert(normals);
    let material = Material {
        shader,
        uniforms: UniformsSet::new(move |mut uniforms| {
            // Set the textures first
            uniforms.set_bundled_texture2d("diffuse_m", &diffuse);
            uniforms.set_bundled_texture2d("normal_m", &normals);
            // Then the parameters
            uniforms.set_f32("bumpiness", 2.3);
            uniforms.set_vec2f32("uv_scale", vek::Vec2::broadcast(0.01));
        }),
    };
    let material = world.pipeline.insert(material);
    // Create some terrain settings
    let terrain_settings = TerrainSettings {
        voxel_src_path: "user/shaders/voxel_terrain/voxel.func.glsl".to_string(),
        depth: 12,
        material,
        ..Default::default()
    };
    let mut terrain = globals::Terrain::new(&world.settings.terrain, terrain_settings, &mut world.pipeline);
    /*
    // Big sphere
    terrain.edit(Edit::sphere(
        vek::Vec3::unit_y() * -50.0,
        50.0,
        EditParams {
            _union: true,
            material: None,
            ..Default::default()
        },
    ));
    */
    world.globals.insert(terrain).unwrap();
}
