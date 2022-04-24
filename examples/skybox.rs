use cflake_engine::{
    assets, defaults,
    defaults::components::{Camera, Light, Transform, Renderer},
    rendering::basics::{lights::LightType, texture::{CubeMap, Texture2D, TextureParams, Texture}, material::{PbrMaterialBuilder, MaterialBuilder}},
    vek, World,
};
// An example with a test camera inside of a skybox
fn main() {
    cflake_engine::start("cflake-examples/skybox", init)
}
// Init the simple camera and the skybox
fn init(world: &mut World) {
    // ----Start the world----
    assets::init!("/examples/assets/");

    defaults::systems::flycam_system::system(world);

    // Create a simple camera entity
    world.ecs.insert(|_, linker| {
        linker.insert(Camera::new(90.0, 2.0, 9000.0)).unwrap();
        linker.insert(Transform::default()).unwrap();
    });

    // Create the directional light source
    world.ecs.insert(|_, linker| {
        let light = Light(LightType::new_directional(1.0, vek::Rgb::one()));
        linker.insert(light).unwrap();
        linker.insert(Transform::rotation_x(-90f32.to_radians())).unwrap();
    });

    // Load the HDR and convert it into a skybox
    let hdr = world.pipeline.insert(assets::load_with::<Texture2D>("defaults/hdr/frozen_lake_4k.hdr", TextureParams::HDR_MAP_LOAD).unwrap());
    let material = PbrMaterialBuilder::default()
        .diffuse(hdr)
        .build(&mut world.pipeline);
    // Create a mesh
    world.ecs.insert(|_, linker| {
        linker.insert(Renderer::new(world.pipeline.defaults().cube.clone(), material)).unwrap();
        linker.insert(Transform::at_y(2.5).scaled_by(vek::Vec3::one() * 5.0)).unwrap();
    });
    
        //println!("{}", hdr.dimensions());
}
