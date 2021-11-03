use super::super::components;
use ecs::{Entity, FilteredLinkedComponents};
use input::*;
use systems::{System, SystemData, SystemEventType};
use world_data::WorldData;

// Events
pub fn entity_update(_system_data: &mut SystemData, _entity: &Entity, components: &FilteredLinkedComponents, data: &mut WorldData) {
    // Rotate the camera around
    let mouse_pos = data.input_manager.get_accumulated_mouse_position();
    const SENSIVITY: f32 = 0.001;
    let new_rotation = veclib::Quaternion::<f32>::from_euler_angles(
        veclib::EulerAnglesOrder::YXZ,
        veclib::Vector3::new(-mouse_pos.1 as f32 * SENSIVITY, -mouse_pos.0 as f32 * SENSIVITY, 0.0),
    );

    // Keyboard input
    let forward = new_rotation.mul_point(veclib::Vector3::<f32>::Z);
    let up = new_rotation.mul_point(veclib::Vector3::<f32>::Y);
    let right = new_rotation.mul_point(veclib::Vector3::<f32>::X);
    let mut velocity: veclib::Vector3<f32> = veclib::Vector3::ZERO;

    // Custom speed
    let original_speed = 1.0 + data.input_manager.get_accumulated_mouse_scroll() * 0.4;
    let speed = original_speed.abs().powf(2.0) * original_speed.signum();

    // Actually update the velocity
    let delta = data.time_manager.delta_time as f32;
    // Forward and backward
    if data.input_manager.map_held("camera_forward").0 {
        velocity += -forward * delta * speed;
    } else if data.input_manager.map_held("camera_backwards").0 {
        velocity += forward * delta * speed;
    }
    // Left and right
    if data.input_manager.map_held("camera_right").0 {
        velocity += right * delta * speed;
    } else if data.input_manager.map_held("camera_left").0 {
        velocity += -right * delta * speed;
    }
    // Up and down
    if data.input_manager.map_held("camera_up").0 {
        velocity += up * delta * speed;
    } else if data.input_manager.map_held("camera_down").0 {
        velocity += -up * delta * speed;
    }

    // Clone first
    let mut position: veclib::Vector3<f32>;
    let mut rotation: veclib::Quaternion<f32>;
    {
        let transform = components.get_component_mut::<components::Transform>(data.component_manager).unwrap();
        position = transform.position;
    }
    // Update the variables
    let physics = components.get_component_mut::<components::Physics>(data.component_manager).unwrap();
    physics.object.linear.velocity = velocity;
    rotation = new_rotation;
    // Update the physics update so we have the velocity applied to the position
    physics.object.update(&mut position, &mut rotation, 1.0);
    // Re-apply
    {
        let transform = components.get_component_mut::<components::Transform>(data.component_manager).unwrap();
        transform.position = position;
        transform.rotation = rotation;
    }
    // Update the matrices
    let camera = components.get_component_mut::<components::Camera>(data.component_manager).unwrap();
    camera.update_view_matrix(position, rotation);
    camera.update_projection_matrix(&data.custom_data.window);
    if !data.input_manager.map_toggled("cull_update") { camera.update_frustum_culling_matrix(); }
}

// Create the camera system
pub fn system(data: &mut WorldData) -> System {
    let mut system = System::new();
    // Link the components
    system.link_component::<components::Camera>(data.component_manager).unwrap();
    system.link_component::<components::Physics>(data.component_manager).unwrap();
    system.link_component::<components::Transform>(data.component_manager).unwrap();
    // Create the inputs
    data.input_manager.bind_key(Keys::W, "camera_forward", MapType::Button);
    data.input_manager.bind_key(Keys::S, "camera_backwards", MapType::Button);
    data.input_manager.bind_key(Keys::D, "camera_right", MapType::Button);
    data.input_manager.bind_key(Keys::A, "camera_left", MapType::Button);
    data.input_manager.bind_key(Keys::Space, "camera_up", MapType::Button);
    data.input_manager.bind_key(Keys::LeftShift, "camera_down", MapType::Button);
    data.input_manager.bind_key(Keys::RightShift, "cull_update", MapType::Toggle);
    // Attach the events
    system.event(SystemEventType::EntityUpdate(entity_update));
    system
}
