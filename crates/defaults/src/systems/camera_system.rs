use super::super::components;
use ecs::{Entity, FilteredLinkedComponents};
use input::*;
use world_data::WorldData;
use systems::{System, SystemData, SystemEventType};


// Events
pub fn entity_update(system_data: &mut SystemData, entity: &Entity, components: &FilteredLinkedComponents, data: &mut WorldData) {   
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
    if data.input_manager.map_held("camera_forward").0 {
        velocity = -forward * delta * speed;
    } else if data.input_manager.map_held("camera_backwards").0 {
        velocity = forward * delta * speed;
    }
    if data.input_manager.map_held("camera_right").0 {
        velocity = right * delta * speed;
    } else if data.input_manager.map_held("camera_left").0 {
        velocity = -right * delta * speed;
    }
    if data.input_manager.map_held("camera_up").0 {
        velocity = up * delta * speed;
    } else if data.input_manager.map_held("camera_down").0 {
        velocity = -up * delta * speed;
    }

    // Update the variables
    let physics =components.get_component_mut::<components::Physics>(data.component_manager).unwrap();
    physics.object.linear.velocity = velocity;
    physics.object.angular.rotation = new_rotation;
    // Update the physics update so we have the velocity applied to the position
    physics.object.update();
    // Update the matrices
    let physics = components.get_component::<components::Physics>(data.component_manager).unwrap();
    let (position, rotation) = (physics.object.linear.position, physics.object.angular.rotation);
    let camera = components.get_component_mut::<components::Camera>(data.component_manager).unwrap();
    camera.update_view_matrix(position, rotation);
    camera.update_projection_matrix(&data.custom_data.window);
    camera.update_frustum_culling_matrix();
}

// Create the camera system
pub fn system(data: &mut WorldData) -> System {
    let mut system = System::new();
    // Link the components
    system.link_component::<components::Camera>(data.component_manager).unwrap();
    system.link_component::<components::Physics>(data.component_manager).unwrap();
    // Create the inputs
    data.input_manager.bind_key(Keys::W, "camera_forward", MapType::Button);
    data.input_manager.bind_key(Keys::S, "camera_backwards", MapType::Button);
    data.input_manager.bind_key(Keys::D, "camera_right", MapType::Button);
    data.input_manager.bind_key(Keys::A, "camera_left", MapType::Button);
    data.input_manager.bind_key(Keys::Space, "camera_up", MapType::Button);
    data.input_manager.bind_key(Keys::LeftShift, "camera_down", MapType::Button);
    // Attach the events
    system.event(SystemEventType::EntityUpdate(entity_update));
    system
}
