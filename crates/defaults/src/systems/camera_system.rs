use world::ecs::component::{ComponentQueryParameters, ComponentQuerySet};
use world::input::Keys;
use world::World;

// The camera system update loop
fn run(world: &mut World, mut data: ComponentQuerySet) {
    let query = data.get_mut(0).unwrap();

    // Set the main camera entity key in the world global
    let global = world.globals.get_mut::<crate::globals::GlobalWorldData>().unwrap();
    // If there isn't a main camera assigned already, we can be the main one
    if let Some((&key, _)) = query.delta.added.iter().next() {
        global.main_camera = key;
    }

    // Rotate the camera around
    let mouse_pos = *world.input.get_mouse_position();
    const SENSIVITY: f32 = 0.0007;
    // Create the camera rotation quaternion
    let new_rotation = veclib::Quaternion::<f32>::from_euler_angles(
        veclib::EulerAnglesOrder::YXZ,
        veclib::Vector3::new(-mouse_pos.y as f32 * SENSIVITY, -mouse_pos.x as f32 * SENSIVITY, 0.0),
    );
    // Calculate the vectors
    let forward = new_rotation.mul_point(-veclib::Vector3::<f32>::Z);
    let up = new_rotation.mul_point(veclib::Vector3::<f32>::Y);
    let right = new_rotation.mul_point(veclib::Vector3::<f32>::X);
    let mut velocity: veclib::Vector3<f32> = veclib::Vector3::ZERO;

    // Custom speed
    let original_speed = 0.1 + (*world.input.get_mouse_scroll() as f32 * 0.1).clamp(0.0, 100.0).powf(2.0);
    let speed = original_speed.abs().powf(2.0) * original_speed.signum() * 1.0 * world.time.delta as f32;
    let fov_delta = if world.input.map_held("camera_zoom") {
        1.0
    } else if world.input.map_held("camera_unzoom") {
        -1.0
    } else {
        0.0
    } * world.time.delta as f32
        * 10.0;

    // Actually update the velocity
    // Forward and backward
    if world.input.map_held("camera_forward") {
        velocity += forward * speed;
    } else if world.input.map_held("camera_backwards") {
        velocity += -forward * speed;
    }
    // Left and right
    if world.input.map_held("camera_right") {
        velocity += right * speed;
    } else if world.input.map_held("camera_left") {
        velocity += -right * speed;
    }
    // Up and down
    if world.input.map_held("camera_up") {
        velocity += up * speed;
    } else if world.input.map_held("camera_down") {
        velocity += -up * speed;
    }
    // Update the camera values now
    let global = world.globals.get::<crate::globals::GlobalWorldData>().unwrap();
    for (&key, components) in query.all.iter_mut() {
        // If we are not the right camera, skip
        if key != global.main_camera {
            continue;
        }
        let mut transform = components.get_mut::<crate::components::Transform>().unwrap();
        transform.position += velocity;
        transform.rotation = new_rotation;
        let (position, _rotation) = (transform.position, transform.rotation);
        let mut camera = components.get_mut::<crate::components::Camera>().unwrap();
        camera.horizontal_fov += fov_delta;

        // Calculate aspect ratio
        let ratio = world.pipeline.window.dimensions().x as f32 / world.pipeline.window.dimensions().y as f32;

        // And don't forget to update the camera matrices
        camera.update_projection_matrix(ratio);
        camera.update_view_matrix(position, new_rotation);
    }
}

// Create the camera system
pub fn system(world: &mut World) {
    world
        .ecs
        .systems
        .builder()
        .event(run)
        .query(
            ComponentQueryParameters::default()
                .link::<crate::components::Camera>()
                .link::<crate::components::Transform>(),
        )
        .build();
    world.input.bind_key(Keys::W, "camera_forward");
    world.input.bind_key(Keys::S, "camera_backwards");
    world.input.bind_key(Keys::D, "camera_right");
    world.input.bind_key(Keys::A, "camera_left");
    world.input.bind_key(Keys::Space, "camera_up");
    world.input.bind_key(Keys::LShift, "camera_down");
    world.input.bind_key(Keys::Z, "camera_zoom");
    world.input.bind_key(Keys::X, "camera_unzoom");
    world.input.bind_key(Keys::RShift, "cull_update");
}
