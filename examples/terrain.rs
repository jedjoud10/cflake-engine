use cflake_engine::{
    assets,
    defaults::{
        components::{Camera, Light, Transform},
        resources::{self, TerrainSettings},
    },
    rendering::basics::{
        lights::LightType::{self},
        material::Material,
        shader::{Shader, ShaderInitSettings},
        texture::{bundle, Texture2D, TextureParams},
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
        let light = Light(LightType::new_directional(4.5, vek::Rgb::one()));
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
    let diffuse = TextureParams::NORMAL_MAP_LOAD;
    let normal = TextureParams::NORMAL_MAP_LOAD;
    let texture_diff_1 = assets::load_with::<Texture2D>("user/textures/Snow006_2K_Color.jpg", diffuse).unwrap();
    let texture_norm_1 = assets::load_with::<Texture2D>("user/textures/Snow006_2K_NormalGL.jpg", normal).unwrap();
    let texture_diff_2 = assets::load_with::<Texture2D>("user/textures/rocks_ground_06_diff_2k.jpg", diffuse).unwrap();
    let texture_norm_2 = assets::load_with::<Texture2D>("user/textures/rocks_ground_06_nor_gl_2k.jpg", normal).unwrap();
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
            uniforms.set_f32("bumpiness", 1.0);
            uniforms.set_vec2f32("uv_scale", vek::Vec2::broadcast(0.01));
        }),
    };
    let material = world.pipeline.insert(material);
    // Create some terrain settings
    let terrain_settings = TerrainSettings {
        voxel_src_path: "user/shaders/voxel_terrain/voxel.func.glsl".to_string(),
        depth: 6,
        material,
        ..Default::default()
    };
    let mut terrain = resources::Terrain::new(terrain_settings, &mut world.pipeline);
    // Big sphere
    terrain.edit(Edit::sphere(vek::Vec3::unit_y() * -50.0, 50.0, EditParams::new(None, vek::Rgb::one(), true)));
    world.resources.insert(terrain).unwrap();
}
