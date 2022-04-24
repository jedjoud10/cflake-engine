use cflake_engine::{
    assets, defaults,
    defaults::components::{Camera, Light, Renderer, Transform},
    rendering::basics::{lights::LightType, material::{PbrMaterialBuilder, MaterialBuilder}, texture::{TextureParams, TextureLayout, TextureFilter, TextureWrapMode, TextureFlags, Texture2D}},
    vek, World,
};
// An example with multiple meshes in the same scene
fn main() {
    cflake_engine::start("cflake-examples/batch", init)
}
// Init the simple camera and multiple meshes
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
        linker.insert(Transform::rotation_z(-90f32.to_radians())).unwrap();
    });

    // Mask that is 100% smooth, 100% metallic
    let mask = Texture2D::new(vek::Extent2::one(), Some(vec![255, 255, 255, 0]), TextureParams {
        layout: TextureLayout::LOADED,
        filter: TextureFilter::Nearest,
        wrap: TextureWrapMode::Repeat,
        flags: TextureFlags::MIPMAPS,
    });
    let mask = world.pipeline.insert(mask);

    // Create multiple cubes
    let cube = world.pipeline.defaults().cube.clone();
    for x in 0..10 {
        for y in 0..10 {

            // Create a material with unique roughness / metallic
            let material = PbrMaterialBuilder::default()
                .mask(mask.clone())
                .tint(vek::Rgb::blue())
                .metallic(x as f32 / 10.0)
                .roughness(y as f32 / 10.0)
                .build(&mut world.pipeline);

            world.ecs.insert(|_, linker| {
                linker.insert(Renderer::new(cube.clone(), material)).unwrap();
                linker.insert(Transform::from((x as f32 * 2.0, y as f32 * 2.0, 0.0))).unwrap();
            });
        }
    }
}
