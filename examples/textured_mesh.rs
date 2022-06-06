/*
use cflake_engine::{
    assets, defaults,
    defaults::components::{Camera, Light, Renderer, Transform},
    rendering::basics::{
        lights::LightType,
        material::{Material, PbrMaterial},
        mesh::Mesh,
        texture::{Texture2D, TextureFilter, TextureParams},
    },
    vek, World,
};
// An example with a test mesh
fn main() {
    cflake_engine::start("cflake-examples/textured-mesh", init)
}
// Init the simple camera and simple mesh
fn init(world: &mut World) {
    // ----Start the world----
    assets::init!("/examples/assets/");

    defaults::systems::flycam_system::system(world);

    // Create a simple camera entity
    world.ecs.insert(|_, linker| {
        linker.insert(Camera::new(90.0, 0.2, 9000.0)).unwrap();
        linker.insert(Transform::default()).unwrap();
    });

    // Create the directional light source
    world.ecs.insert(|_, linker| {
        let light = Light(LightType::directional(vek::Rgb::one() * 6.0));
        linker.insert(light).unwrap();
        linker.insert(Transform::rotation_x(-20f32.to_radians())).unwrap();
    });

    // Simple material
    let floor = world.pipeline.insert(Material::new(PbrMaterial::from(vek::Rgb::white())));

    // Load a diffuse map
    let diff = assets::load_with::<Texture2D>("user/textures/rocks_ground_06_diff_4k.jpg", TextureParams::DIFFUSE_MAP_LOAD).unwrap();
    let diff = world.pipeline.insert(diff);

    // Load a mask map
    let mask = assets::load_with::<Texture2D>("user/textures/rocks_ground_06_arm_4k.jpg", TextureParams::NON_COLOR_MAP_LOAD).unwrap();
    let mask = world.pipeline.insert(mask);

    // Load a normal map
    let norm = assets::load_with::<Texture2D>("user/textures/rocks_ground_06_nor_gl_4k.jpg", TextureParams::NON_COLOR_MAP_LOAD).unwrap();
    let norm = world.pipeline.insert(norm);

    // AAA
    let material = Material::new(PbrMaterial::default().diffuse(diff).normal(norm).mask(mask).roughness(0.2));
    let material = world.pipeline.insert(material);

    // Create a mesh
    world.ecs.insert(|_, linker| {
        linker.insert(Renderer::new(world.pipeline.defaults().cube.clone(), material)).unwrap();
        linker.insert(Transform::at_y(2.5).scaled_by(vek::Vec3::one() * 5.0)).unwrap();
    });

    // Create a floor
    let plane = world.pipeline.defaults().plane.clone();
    world.ecs.insert(|_, linker| {
        linker.insert(Renderer::new(plane, floor)).unwrap();
        linker.insert(Transform::default().scaled_by(vek::Vec3::new(10.0, 1.0, 10.0))).unwrap();
    });
}
*/

fn main() {}
