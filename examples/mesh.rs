use cflake_engine::prelude::*;

// Example that will try to render a simple 3D cube
fn main() {
    App::default().insert_system(system).execute();
}

fn system(events: &Events) {
    events.register::<Init>(init);
}

// Initialize the world
fn init(world: &mut World) {
    // ---- Initialize the world ---- \\
    let (ecs, graphic, assets) = world
        .get_mut::<(&mut EcsManager, &mut Graphics, &mut Assets)>()
        .unwrap();
}
/*
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
        linker.insert(Transform::rotation_x(-45f32.to_radians())).unwrap();
    });

    // Create a sphere
    let sphere = world.pipeline.defaults().sphere.clone();
    world.ecs.insert(|_, linker| {
        linker.insert(Renderer::from(sphere)).unwrap();
        linker.insert(Transform::at_y(0.5)).unwrap();
    });

    // Create a floor
    let plane = world.pipeline.defaults().plane.clone();
    world.ecs.insert(|_, linker| {
        linker.insert(Renderer::from(plane)).unwrap();
        linker.insert(Transform::default().scaled_by(vek::Vec3::new(10.0, 1.0, 10.0))).unwrap();
    });
}
*/
