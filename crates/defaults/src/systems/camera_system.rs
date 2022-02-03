use main::core::{Context, WriteContext};
use main::ecs::event::EventKey;
use main::input::Keys;

// The camera system update loop
fn run(context: &mut Context, data: EventKey) {
    let (query, mut global_fetcher) = data.decompose().unwrap();
    let mut write = context.write().unwrap();
    // Rotate the camera around
    let mouse_pos = write.input.get_mouse_position();
    const SENSIVITY: f32 = 0.001;
    // Create the camera rotation quaternion
    let new_rotation = veclib::Quaternion::<f32>::from_euler_angles(
        veclib::EulerAnglesOrder::YXZ,
        veclib::Vector3::new(-mouse_pos.1 as f32 * SENSIVITY, -mouse_pos.0 as f32 * SENSIVITY, 0.0),
    );
    // Calculate the vectors
    let forward = new_rotation.mul_point(veclib::Vector3::<f32>::Z);
    let up = new_rotation.mul_point(veclib::Vector3::<f32>::Y);
    let right = new_rotation.mul_point(veclib::Vector3::<f32>::X);
    let mut velocity: veclib::Vector3<f32> = veclib::Vector3::ZERO;

    // Custom speed
    let original_speed = 0.1 + (write.input.get_mouse_scroll() as f32 * 0.1).clamp(0.0, 100.0).powf(2.0);
    let speed = original_speed.abs().powf(2.0) * original_speed.signum() * 1.0 * write.time.delta as f32;
    let fov_delta = if write.input.map_held("camera_zoom") {
        1.0
    } else if write.input.map_held("camera_unzoom") {
        -1.0
    } else {
        0.0
    } * write.time.delta as f32
        * 10.0;

    // Actually update the velocity
    // Forward and backward
    if write.input.map_held("camera_forward") {
        velocity += -forward * speed;
    } else if write.input.map_held("camera_backwards") {
        velocity += forward * speed;
    }
    // Left and right
    if write.input.map_held("camera_right") {
        velocity += right * speed;
    } else if write.input.map_held("camera_left") {
        velocity += -right * speed;
    }
    // Up and down
    if write.input.map_held("camera_up") {
        velocity += up * speed;
    } else if write.input.map_held("camera_down") {
        velocity += -up * speed;
    }
    // Update the camera values now
    query.update_all(move |linked_components| {
        let mut transform = linked_components.get_component_mut::<crate::components::Transform>().unwrap();
        transform.position += velocity;
        transform.rotation = new_rotation;
        let (position, rotation) = (transform.position, transform.rotation);
        let mut camera = linked_components.get_component_mut::<crate::components::Camera>().unwrap();
        camera.horizontal_fov += fov_delta;
        // And don't forget to update the camera matrices
        // Load the pipeline since we need to get the window settings
        let pipeline = write.pipeline.read();
        camera.update_aspect_ratio(pipeline.window.dimensions);
        camera.update_view_matrix(position, new_rotation);

        use main::rendering::object;
        use main::rendering::pipeline;
        let pipeline_camera = main::rendering::pipeline::camera::Camera {
            position,
            rotation,
            viewm: camera.view_matrix,
            projm: camera.projection_matrix,
            clip_planes: camera.clip_planes,
        };
        pipeline::pipec::task(object::PipelineTask::UpdateCamera(pipeline_camera), &*pipeline);
        drop(pipeline);

        // If we are the main camera, we must update our position in the global
        let mut global = write.ecs.get_global_mut::<crate::globals::GlobalWorldData>(&mut global_fetcher).unwrap();
        global.camera_pos = position;
        global.camera_dir = rotation.mul_point(veclib::Vector3::Z);
    })
}

// Create the camera system
pub fn system(write: &mut WriteContext) {
    write
        .ecs
        .create_system_builder()
        .with_run_event(run)
        .link::<crate::components::Camera>()
        .link::<crate::components::Transform>()
        .build();
    write.input.bind_key(Keys::W, "camera_forward");
    write.input.bind_key(Keys::S, "camera_backwards");
    write.input.bind_key(Keys::D, "camera_right");
    write.input.bind_key(Keys::A, "camera_left");
    write.input.bind_key(Keys::Space, "camera_up");
    write.input.bind_key(Keys::LeftShift, "camera_down");
    write.input.bind_key(Keys::Z, "camera_zoom");
    write.input.bind_key(Keys::X, "camera_unzoom");
    write.input.bind_key(Keys::RightShift, "cull_update");
}
