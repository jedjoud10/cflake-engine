use cflake_engine::{
    assets,
    defaults,
    defaults::components::{Camera, Light, Transform, Renderer, Collider, ColliderGeometry, ColliderBuilder, RigidBody, RigidBodyType},
    rendering::basics::lights::LightType,
    vek, World,
};
// An example with multiple physics objects
fn main() {
    cflake_engine::start("cflake-examples/camera", init)
}
// Init the physic objects
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

    // Create a flat plane
    let cube = world.pipeline.defaults().cube.clone();
    world.ecs.insert(|_, linker| {
        let transform = Transform::scale_y(0.01).scaled_by(vek::Vec3::one() * 50.0);
        linker.insert(ColliderBuilder::cuboid(transform.scale).build()).unwrap();
        linker.insert(transform).unwrap();
        linker.insert(Renderer::from(cube.clone())).unwrap();
        linker.insert(RigidBody::new(RigidBodyType::Static)).unwrap();
    });

    // Create a few physics cubes
    for x in 0..5 {
        for z in 0..5 {
            for y in 0..20 {
                world.ecs.insert(|_, linker| {
                    linker.insert(ColliderBuilder::cuboid(vek::Vec3::one()).build()).unwrap();
                    linker.insert(Transform::new_xyz(x as f32, y as f32 * 1.2 + 50.0, z as f32)).unwrap();
                    linker.insert(Renderer::from(cube.clone())).unwrap();
                    linker.insert(RigidBody::new(RigidBodyType::Dynamic)).unwrap();
                });
            }
        }
    }



    
}
