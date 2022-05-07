use cflake_engine::{
    assets,
    defaults::{
        components::{Camera, Light, Transform},
        resources::{self, TerrainSettings},
    },
    rendering::basics::{
        lights::LightType::{self},
        material::{Material, MaterialType, PbrMaterial},
        shader::{Shader, ShaderInitSettings},
        texture::{bundle, Texture2D, TextureParams},
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
        linker.insert(Camera::new(90.0, 20.0, 900000.0)).unwrap();
        linker.insert(Transform::default()).unwrap();
    });

    // Create the directional light source
    world.ecs.insert(|_, linker| {
        let light = Light(LightType::directional(vek::Rgb::one() * 6.0));
        linker.insert(light).unwrap();
        linker.insert(Transform::rotation_x(-25f32.to_radians())).unwrap();
    });

    // Load a terrain material
    // Load the shader first
    let settings = ShaderInitSettings::default()
        .source("defaults/shaders/voxel_terrain/terrain.vrsh.glsl")
        .source("user/shaders/voxel_terrain/terrain.frsh.glsl");
    let shader = world.pipeline.insert(Shader::new(settings).unwrap());
    // Then the textures
    let diffuse = TextureParams::DIFFUSE_MAP_LOAD;
    let other = TextureParams::NON_COLOR_MAP_LOAD;
    let diffuse1 = assets::load_with::<Texture2D>("user/textures/rocks_ground_06_diff_4k.jpg", diffuse).unwrap();
    let normal1 = assets::load_with::<Texture2D>("user/textures/rocks_ground_06_nor_gl_4k.jpg", other).unwrap();
    let mask1 = assets::load_with::<Texture2D>("user/textures/rocks_ground_06_arm_4k.jpg", other).unwrap();
    let diffuse1 = world.pipeline.insert(diffuse1);
    let normal1 = world.pipeline.insert(normal1);
    let mask1 = world.pipeline.insert(mask1);
    let material = Material::from_parts(
        shader,
        PbrMaterial::default().diffuse(diffuse1).normal(normal1).mask(mask1).scale(vek::Vec2::broadcast(0.01)),
    );
    let material = world.pipeline.insert(material);
    // Create some terrain settings
    let terrain_settings = TerrainSettings {
        voxel_src_path: "user/shaders/voxel_terrain/voxel.func.glsl".to_string(),
        depth: 9,
        material,
        ..Default::default()
    };
    let mut terrain = resources::Terrain::new(terrain_settings, &mut world.pipeline);
    // Big sphere
    terrain.edit(Edit::sphere(vek::Vec3::unit_y() * -50.0, 50.0, EditParams::new(None, vek::Rgb::one(), true)));
    world.resources.insert(terrain).unwrap();
}
