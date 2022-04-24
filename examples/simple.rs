use cflake_engine::{
    assets,
    defaults::{
        components::{Camera, Light, Transform},
        resources::{self, TerrainSettings},
    },
    rendering::basics::{
        lights::LightType::{self},
        material::{Material, PbrMaterialBuilder, MaterialBuilder},
        shader::{Shader, ShaderInitSettings},
        texture::{bundle, Texture2D, TextureParams},
        uniforms::UniformsSet,
    },
    terrain::editing::{Edit, EditParams},
    vek, World,
};

// A game with some simple test terrain
fn main() {
    cflake_engine::start("cflake-examples/simple", init)
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
        let light = Light(LightType::new_directional(0.9, vek::Rgb::one()));
        linker.insert(light).unwrap();
        linker.insert(Transform::rotation_x(-45f32.to_radians())).unwrap();
    });

    // A simple material with a specific color
    let color = vek::Rgb::cyan(); 
    let flat = world.pipeline.defaults().flat.clone();
    let material = PbrMaterialBuilder::default().tint(color).build_with_shader(&mut world.pipeline, flat);
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