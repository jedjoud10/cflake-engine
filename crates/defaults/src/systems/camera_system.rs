use ecs::component::*;
use main::core::{Context, WriteContext};
use main::ecs;
use main::input::{Keys, MapType};

// The camera system update loop
fn run(context: Context, components: ComponentQuery) {
    let read = context.read();
    // Rotate the camera around
    let mouse_pos = read.input.get_accumulated_mouse_position();
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
    let original_speed = 1.0 + (read.input.get_accumulated_mouse_scroll() * 0.1).powf(2.0);
    let speed = original_speed.abs().powf(2.0) * original_speed.signum() * 1.0 * read.time.delta as f32;

    // Actually update the velocity
    // Forward and backward
    if read.input.map_held("camera_forward").0 {
        velocity += -forward * speed;
    } else if read.input.map_held("camera_backwards").0 {
        velocity += forward * speed;
    }
    // Left and right
    if read.input.map_held("camera_right").0 {
        velocity += right * speed;
    } else if read.input.map_held("camera_left").0 {
        velocity += -right * speed;
    }
    // Up and down
    if read.input.map_held("camera_up").0 {
        velocity += up * speed;
    } else if read.input.map_held("camera_down").0 {
        velocity += -up * speed;
    }
    // Update the camera values now
    components.update_all(move |linked_components| {
        let mut transform = linked_components.component_mut::<crate::components::Transform>().unwrap();
        transform.position += velocity;
        transform.rotation = new_rotation;
        let (position, rotation) = (transform.position, transform.rotation);
        let mut camera = linked_components.component_mut::<crate::components::Camera>().unwrap();
        // And don't forget to update the camera matrices
        let world = &*read;
        // Load the pipeline since we need to get the window settings
        let pipeline = world.pipeline.read().unwrap();
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
    })
}

// Create the camera system
pub fn system(write: &mut WriteContext) {
    write
        .ecs
        .create_system_builder()
        .set_run_event(run)
        .link::<crate::components::Camera>()
        .link::<crate::components::Transform>()
        .build();
    write.input.bind_key(Keys::W, "camera_forward", MapType::Button);
    write.input.bind_key(Keys::S, "camera_backwards", MapType::Button);
    write.input.bind_key(Keys::D, "camera_right", MapType::Button);
    write.input.bind_key(Keys::A, "camera_left", MapType::Button);
    write.input.bind_key(Keys::Space, "camera_up", MapType::Button);
    write.input.bind_key(Keys::LeftShift, "camera_down", MapType::Button);
    write.input.bind_key(Keys::RightShift, "cull_update", MapType::Toggle);
}