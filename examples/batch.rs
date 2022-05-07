use cflake_engine::{
    assets, defaults,
    defaults::components::{Camera, Light, Renderer, Transform},
    rendering::basics::{
        lights::LightType,
        material::{MaskBuilder, Material, PbrMaterial},
        texture::{Texture2D, TextureFilter, TextureFlags, TextureLayout, TextureParams, TextureWrapMode},
    },
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
        let light = Light(LightType::directional(vek::Rgb::one() * 6.0));
        linker.insert(light).unwrap();
        linker.insert(Transform::rotation_x(-30f32.to_radians())).unwrap();
    });

    // Mask that is 100% smooth, 100% metallic
    let mask = Texture2D::new(
        vek::Extent2::one(),
        Some(vec![255, 255, 255, 0]),
        TextureParams {
            layout: TextureLayout::LOADED,
            filter: TextureFilter::Nearest,
            wrap: TextureWrapMode::Repeat,
            flags: TextureFlags::MIPMAPS,
        },
    );
    let mask = world.pipeline.insert(mask);

    // Load a diffuse map
    let diff = assets::load_with::<Texture2D>("user/textures/MetalPlates012_4K_Color.jpg", TextureParams::DIFFUSE_MAP_LOAD).unwrap();
    let diff = world.pipeline.insert(diff);

    // Create a mask map
    let ao = assets::load_with::<Texture2D>("user/textures/MetalPlates012_4K_AmbientOcclusion.jpg", TextureParams::NON_COLOR_MAP_LOAD).unwrap();
    let metallic = assets::load_with::<Texture2D>("user/textures/MetalPlates012_4K_Metalness.jpg", TextureParams::NON_COLOR_MAP_LOAD).unwrap();
    let roughness = assets::load_with::<Texture2D>("user/textures/MetalPlates012_4K_Roughness.jpg", TextureParams::NON_COLOR_MAP_LOAD).unwrap();
    /*
    let mask = world.pipeline.insert(mask);
    */

    let mask = MaskBuilder::default().ao(ao).metallic(metallic).roughness(roughness).build().unwrap();
    let mask = world.pipeline.insert(mask);

    // Load a normal map
    let norm = assets::load_with::<Texture2D>("user/textures/MetalPlates012_4K_NormalGL.jpg", TextureParams::NON_COLOR_MAP_LOAD).unwrap();
    let norm = world.pipeline.insert(norm);

    // Create multiple sphere
    let sphere = world.pipeline.defaults().cube.clone();
    for x in 0..11 {
        for y in 0..11 {
            // Create a material with unique roughness / metallic
            let material = world.pipeline.insert(Material::new(PbrMaterial {
                diffuse: diff.clone(),
                normal: norm.clone(),
                mask: mask.clone(),
                roughness: y as f32 / 10.0,
                metallic: x as f32 / 10.0,
                ..Default::default()
            }));

            world.ecs.insert(|_, linker| {
                linker.insert(Renderer::new(sphere.clone(), material)).unwrap();
                linker
                    .insert(Transform::from((x as f32 * 10.0, y as f32 * 10.0, 0.0)).scaled_by(vek::Vec3::one() * 5.0))
                    .unwrap();
            });
        }
    }

    // Rotate each valid renderer entity
    world.events.insert(|world| {
        for (i, (transform, _)) in world.ecs.try_query::<(&mut Transform, &Renderer)>().enumerate() {
            transform.rotation = transform.rotation
                * vek::Quaternion::rotation_x(0.06 * world.time.delta() * (i as f32).sin())
                * vek::Quaternion::rotation_z(-0.06 * world.time.delta() * (i as f32).cos());
        }
    })
}
