use cflake_engine::{
    assets,
    defaults,
    defaults::components::{Camera, Light, Renderer, Transform},
    rendering::basics::{
        lights::LightType,
        material::{MaterialBuilder, PbrMaterialBuilder},
        mesh::Mesh,
        texture::{Texture2D, TextureParams, TextureFilter},
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
        linker.insert(Transform::rotation_x(-20f32.to_radians())).unwrap();
    });

        // Simple material
        let floor = PbrMaterialBuilder::default().tint(vek::Rgb::white()).build(&mut world.pipeline);
    
        let norm = assets::load_with::<Texture2D>("user/textures/debug.png", TextureParams {
            filter: TextureFilter::Nearest,
            ..TextureParams::NORMAL_MAP_LOAD
        }).unwrap();
        let norm = world.pipeline.insert(norm);
        let material = PbrMaterialBuilder::default()
            .diffuse(world.pipeline.defaults().white.clone())
            .normal(norm)
            .build(&mut world.pipeline);
    
        // Create a cube
        let cube = world.pipeline.defaults().cube.clone();
        world.ecs.insert(|_, linker| {
            linker.insert(Renderer::new(cube, material)).unwrap();
            linker.insert(Transform::at_y(0.5)).unwrap();
        });
    
        // Create a floor
        let plane = world.pipeline.defaults().plane.clone();
        world.ecs.insert(|_, linker| {
            linker.insert(Renderer::new(plane, floor)).unwrap();
            linker.insert(Transform::default().scaled_by(vek::Vec3::new(10.0, 1.0, 10.0))).unwrap();
        });
}
