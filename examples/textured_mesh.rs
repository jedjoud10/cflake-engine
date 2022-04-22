use cflake_engine::{
    assets::{self, assetc},
    defaults,
    defaults::components::{Camera, Light, Renderer, Transform},
    rendering::basics::{
        lights::LightType,
        material::{MaterialBuilder, PbrMaterialBuilder},
        mesh::Mesh,
        texture::{Texture2D, TextureParams},
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
        let light = Light(LightType::new_directional(1.0, vek::Rgb::one()));
        linker.insert(light).unwrap();
        linker.insert(Transform::rotation_x(-90f32.to_radians())).unwrap();
    });

    // Simple material with textures
    let mesh = assetc::load::<Mesh>("user/meshes/untitled.obj").unwrap();
    let mesh = world.pipeline.insert(mesh);
    let diff = assetc::load_with::<Texture2D>("user/textures/rocks_ground_06_diff_2k.jpg", TextureParams::DIFFUSE_MAP_LOAD).unwrap();
    let norm = assetc::load_with::<Texture2D>("user/textures/rocks_ground_06_nor_gl_2k.jpg", TextureParams::NORMAL_MAP_LOAD).unwrap();
    let diff = world.pipeline.insert(diff);
    let norm = world.pipeline.insert(norm);
    let material = PbrMaterialBuilder::default()
        .diffuse(diff)
        .normal(norm)
        .bumpiness(1.0)
        .scale(vek::Vec2::one() * 8.0)
        .build(&mut world.pipeline);

    // Create an entity
    world.ecs.insert(|_, linker| {
        linker.insert(Renderer::new(mesh.clone(), material)).unwrap();
        linker.insert(Transform::rotation_x(2.0)).unwrap();
    });
}
