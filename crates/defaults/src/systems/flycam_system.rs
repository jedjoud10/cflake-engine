use world::input::Keys;
use world::World;

use crate::components::{Camera, Transform};
use crate::globals::GlobalWorldData;

// Move the main camera around
fn run(world: &mut World) {
    if !world.input.is_accepting_input() {
        return;
    }
    // Rotate the camera around
    let mouse_pos = world.input.mouse_pos();
    const SENSIVITY: f32 = 0.0007;
    // Create the camera rotation quaternion
    let new_rotation = vek::Quaternion::rotation_y(-mouse_pos.x as f32 * SENSIVITY) * vek::Quaternion::rotation_x(-mouse_pos.y as f32 * SENSIVITY);
    let mat = vek::Mat4::from(new_rotation);
    // Calculate the vectors
    let forward = mat.mul_direction(-vek::Vec3::<f32>::unit_z());
    let up = mat.mul_direction(vek::Vec3::<f32>::unit_y());
    let right = mat.mul_direction(vek::Vec3::<f32>::unit_x());
    let mut velocity: vek::Vec3<f32> = vek::Vec3::zero();

    // Custom speed
    let original_speed = 0.1 + (world.input.mouse_scroll() as f32 * 0.1).clamp(0.0, 100.0).powf(2.0);
    let speed = original_speed.abs().powf(2.0) * original_speed.signum() * 1.0 * world.time.delta() as f32;
    let fov_delta = if world.input.held("camera_zoom") {
        1.0
    } else if world.input.held("camera_unzoom") {
        -1.0
    } else {
        0.0
    } * world.time.delta() as f32
        * 10.0;

    // Actually update the velocity
    // Forward and backward
    if world.input.held("camera_forward") {
        velocity += forward * speed;
    } else if world.input.held("camera_backwards") {
        velocity += -forward * speed;
    }
    // Left and right
    if world.input.held("camera_right") {
        velocity += right * speed;
    } else if world.input.held("camera_left") {
        velocity += -right * speed;
    }
    // Up and down
    if world.input.held("camera_up") {
        velocity += up * speed;
    } else if world.input.held("camera_down") {
        velocity += -up * speed;
    }

    // Update the camera values now
    let global = world.globals.get::<GlobalWorldData>().unwrap();

    // Fetch the main camera
    let entry = world.ecs.entry(global.camera);
    if let Some(mut entry) = entry {
        // Get the transform and update it
        let transform = entry.get_mut::<Transform>().unwrap();
        transform.position += velocity;
        transform.rotation = new_rotation;

        // Get the camera and update it
        let camera = entry.get_mut::<Camera>().unwrap();
        camera.horizontal_fov += fov_delta;
    }
}

// Create the flycam system
pub fn system(world: &mut World) {
    world.systems.insert(run);
    world.input.bind(Keys::W, "camera_forward");
    world.input.bind(Keys::S, "camera_backwards");
    world.input.bind(Keys::D, "camera_right");
    world.input.bind(Keys::A, "camera_left");
    world.input.bind(Keys::Space, "camera_up");
    world.input.bind(Keys::LShift, "camera_down");
    world.input.bind(Keys::Z, "camera_zoom");
    world.input.bind(Keys::X, "camera_unzoom");
}
