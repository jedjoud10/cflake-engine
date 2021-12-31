use core::global::callbacks::CallbackType::LocalEntityMut;

use ecs::{SystemData, SystemEventType};
use input::{Keys, MapType};
use others::callbacks::MutCallback;

// Events
fn entity_update(data: &mut SystemData<()>, entity: &ecs::Entity) {
    //println!("{}", entity.name);
    // Rotate the camera around
    let mouse_pos = core::global::input::mouse_pos();
    const SENSIVITY: f32 = 0.001;
    // Keyboard input
    let new_rotation_ = veclib::Quaternion::<f32>::from_euler_angles(
        veclib::EulerAnglesOrder::YXZ,
        veclib::Vector3::new(-mouse_pos.1 as f32 * SENSIVITY, -mouse_pos.0 as f32 * SENSIVITY, 0.0),
    );
    // Calculate the vectors
    let forward = new_rotation_.mul_point(veclib::Vector3::<f32>::Z);
    let up = new_rotation_.mul_point(veclib::Vector3::<f32>::Y);
    let right = new_rotation_.mul_point(veclib::Vector3::<f32>::X);
    let mut velocity: veclib::Vector3<f32> = veclib::Vector3::ZERO;

    // Custom speed
    let original_speed = 1.0 + (core::global::input::mouse_scroll() * 0.1).powf(2.0);
    let speed = original_speed.abs().powf(2.0) * original_speed.signum() * 1.0;

    // Actually update the velocity    
    // Forward and backward
    if core::global::input::map_held("camera_forward").0 {
        velocity += -forward * speed;
    } else if core::global::input::map_held("camera_backwards").0 {
        velocity += forward * speed;
    }
    // Left and right
    if core::global::input::map_held("camera_right").0 {
        velocity += right * speed;
    } else if core::global::input::map_held("camera_left").0 {
        velocity += -right * speed;
    }
    // Up and down
    if core::global::input::map_held("camera_up").0 {
        velocity += up * speed;
    } else if core::global::input::map_held("camera_down").0 {
        velocity += -up * speed;
    }

    // Clone first
    let position = core::global::ecs::component::<crate::components::Transform>(entity).unwrap().position;
    // Update the position and rotation
    let new_rotation = new_rotation_;
    let delta = core::global::timings::delta() as f32;
    let new_position = position + velocity * delta;
    // We can now update the camera rotation and position
    core::global::ecs::entity_mut(
        entity.id,
        LocalEntityMut(MutCallback::new(move |entity| {
            // Update the transform object
            let transform = core::global::ecs::component_mut::<crate::components::Transform>(entity).unwrap();
            transform.position = new_position;
            transform.rotation = new_rotation;
            let camera = core::global::ecs::component_mut::<crate::components::Camera>(entity).unwrap();
            // And don't forget to update the camera matrices
            camera.update_view_matrix(new_position, new_rotation);
            let pos = new_position;
            let rot = new_rotation;
            let data = (pos, rot, camera.clip_planes, camera.projection_matrix);
            rendering::pipec::task(rendering::pipec::RenderTask::CameraDataUpdate(data));
            if !core::global::input::map_toggled("cull_update") {
                camera.update_frustum_culling_matrix();
            }
        }))
        .create(),
    );
}
fn entity_added(data: &mut SystemData<()>, entity: &ecs::Entity) {
    // Initialize the camera
    core::global::ecs::entity_mut(
        entity.id,
        LocalEntityMut(MutCallback::new(|entity| {
            // We can now update the camera
            let camera = core::global::ecs::component_mut::<crate::components::Camera>(entity).unwrap();
            camera.update_projection_matrix();
            // If it is the first camera in the world, we must set it as the main camera
            core::global::main::world_data_mut(|x| x.main_camera_entity_id = Some(entity.id));
        }))
        .create(),
    );
}

// Create the default system
pub fn system() {
    core::global::ecs::add_system((), || {
        // Create a system
        let mut system = ecs::System::new();
        // Link the components
        system.link::<crate::components::Camera>();
        system.link::<crate::components::Transform>();
        core::global::input::bind_key(Keys::W, "camera_forward", MapType::Button);
        core::global::input::bind_key(Keys::S, "camera_backwards", MapType::Button);
        core::global::input::bind_key(Keys::D, "camera_right", MapType::Button);
        core::global::input::bind_key(Keys::A, "camera_left", MapType::Button);
        core::global::input::bind_key(Keys::Space, "camera_up", MapType::Button);
        core::global::input::bind_key(Keys::LeftShift, "camera_down", MapType::Button);
        core::global::input::bind_key(Keys::RightShift, "cull_update", MapType::Toggle);
        // Link the events
        system.event(SystemEventType::EntityUpdate(entity_update));
        system.event(SystemEventType::EntityAdded(entity_added));
        // Return the newly made system
        system
    });
}
