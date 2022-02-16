use main::core::World;
use main::ecs::event::EventKey;
use main::input::Keys;

// The camera system update loop
fn run(world: &mut World, data: EventKey) {
    let mut query = data.get_query().unwrap();
    // Rotate the camera around
    let mouse_pos = *world.input.get_mouse_position();
    const SENSIVITY: f32 = 0.0007;
    // Create the camera rotation quaternion
    let new_rotation = veclib::Quaternion::<f32>::from_euler_angles(
        veclib::EulerAnglesOrder::YXZ,
        veclib::Vector3::new(-mouse_pos.y as f32 * SENSIVITY, -mouse_pos.x as f32 * SENSIVITY, 0.0),
    );
    // Calculate the vectors
    let forward = new_rotation.mul_point(veclib::Vector3::<f32>::Z);
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
        velocity += -forward * speed;
    } else if world.input.map_held("camera_backwards") {
        velocity += forward * speed;
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
    for (_, components) in query.lock().iter_mut() {
        let mut transform = components.get_component_mut::<crate::components::Transform>().unwrap();
        transform.position += velocity;
        transform.rotation = new_rotation;
        let (position, rotation) = (transform.position, transform.rotation);
        let mut camera = components.get_component_mut::<crate::components::Camera>().unwrap();
        camera.horizontal_fov += fov_delta;
        // And don't forget to update the camera matrices
        // Load the pipeline since we need to get the window settings
        let pipeline = world.pipeline.read();
        camera.update_aspect_ratio(pipeline.window.dimensions);
        camera.update_view_matrix(position, new_rotation);

        use main::rendering::pipeline;
        let pipeline_camera = main::rendering::pipeline::camera::Camera {
            position,
            rotation,
            viewm: camera.view_matrix,
            projm: camera.projection_matrix,
            clip_planes: camera.clip_planes,
        };
        pipeline::pipec::update_callback(&pipeline, |pipeline, _| pipeline.set_internal_camera(pipeline_camera));
        drop(pipeline);

        // If we are the main camera, we must update our position in the global
        let mut global = world.globals.get_global_mut::<crate::globals::GlobalWorldData>().unwrap();
        global.camera_pos = position;
        global.camera_dir = rotation.mul_point(veclib::Vector3::Z);
    }
}

// Create the camera system
pub fn system(world: &mut World) {
    world
        .ecs
        .create_system_builder()
        .with_run_event(run)
        .link::<crate::components::Camera>()
        .link::<crate::components::Transform>()
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
