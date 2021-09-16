use super::super::components;
use hypo_ecs::{Entity, FilteredLinkedComponents};
use hypo_input::*;
use hypo_system_event_data::{SystemEventData, SystemEventDataLite};
use hypo_systems::{System, SystemData};
#[derive(Default)]
pub struct CameraSystem {
    pub system_data: SystemData,
}

impl System for CameraSystem {
    // Wrappers around system data
    fn get_system_data(&self) -> &SystemData {
        &self.system_data
    }

    fn get_system_data_mut(&mut self) -> &mut SystemData {
        &mut self.system_data
    }

    // Setup the system
    fn setup_system(&mut self, data: &mut SystemEventData) {
        let system_data = self.get_system_data_mut();
        system_data.link_component::<components::Camera>(data.component_manager).unwrap();
        system_data.link_component::<components::Transform>(data.component_manager).unwrap();

        data.input_manager.bind_key(Keys::W, "camera_forward", MapType::Button);
        data.input_manager.bind_key(Keys::S, "camera_backwards", MapType::Button);
        data.input_manager.bind_key(Keys::D, "camera_right", MapType::Button);
        data.input_manager.bind_key(Keys::A, "camera_left", MapType::Button);
        data.input_manager.bind_key(Keys::Space, "camera_up", MapType::Button);
        data.input_manager.bind_key(Keys::LeftShift, "camera_down", MapType::Button);
        data.input_manager.bind_key(Keys::J, "update_frustum", MapType::Toggle);
    }

    // Called for each entity in the system
    fn fire_entity(&mut self, components: &FilteredLinkedComponents, data: &mut SystemEventData) {
        let position: veclib::Vector3<f32>;
        let rotation: veclib::Quaternion<f32>;
        {
            // Create some movement using user input
            {
                let _delta_time = data.time_manager.delta_time as f32;
                let changed_rotation: veclib::Quaternion<f32>;

                // Rotate the camera around
                let mouse_pos = data.input_manager.get_accumulated_mouse_position();
                let sensitivity = 0.001_f32;
                changed_rotation = veclib::Quaternion::<f32>::from_euler_angles(
                    veclib::EulerAnglesOrder::YXZ,
                    veclib::Vector3::new(-mouse_pos.1 as f32 * sensitivity, -mouse_pos.0 as f32 * sensitivity, 0.0),
                );

                // Keyboard input
                let forward_vector = veclib::Matrix4x4::from_quaternion(&changed_rotation).mul_point(&veclib::Vector3::<f32>::new(0.0, 0.0, 1.0));
                let up_vector = veclib::Matrix4x4::from_quaternion(&changed_rotation).mul_point(&veclib::Vector3::<f32>::new(0.0, 1.0, 0.0));
                let right_vector = veclib::Matrix4x4::from_quaternion(&changed_rotation).mul_point(&veclib::Vector3::<f32>::new(1.0, 0.0, 0.0));
                let mut changed_position = components.get_component_mut::<components::Transform>(data.component_manager).unwrap().position;
                let delta = data.time_manager.delta_time as f32;
                // Default speed
                let original_speed = 1.0 + data.input_manager.get_accumulated_mouse_scroll() * 0.4;
                let speed = original_speed.abs().powf(2.0) * original_speed.signum();
                if data.input_manager.map_held("camera_forward").0 {
                    changed_position -= forward_vector * delta * speed;
                } else if data.input_manager.map_held("camera_backwards").0 {
                    changed_position += forward_vector * delta * speed;
                }
                if data.input_manager.map_held("camera_right").0 {
                    changed_position += right_vector * delta * speed;
                } else if data.input_manager.map_held("camera_left").0 {
                    changed_position -= right_vector * delta * speed;
                }
                if data.input_manager.map_held("camera_up").0 {
                    changed_position += up_vector * delta * speed;
                } else if data.input_manager.map_held("camera_down").0 {
                    changed_position -= up_vector * delta * speed;
                }

                // Update the variables
                components.get_component_mut::<components::Transform>(data.component_manager).unwrap().position = changed_position;
                components.get_component_mut::<components::Transform>(data.component_manager).unwrap().rotation = changed_rotation;
                position = changed_position;
                rotation = changed_rotation;
            }
        }
        let camera_component = components.get_component_mut::<components::Camera>(data.component_manager).unwrap();
        // Update the view matrix every time we make a change
        camera_component.update_view_matrix(position, rotation);
        camera_component.update_projection_matrix(&data.custom_data.window);
        if !data.input_manager.map_toggled("update_frustum") {
            // Update the frustum culling matrix
            camera_component.update_frustum_culling_matrix();
        }
    }

    // When an entity gets added to this system
    fn entity_added(&mut self, entity: &Entity, data: &mut SystemEventDataLite) {
        // First time we initialize the camera, setup the matrices
        let position: veclib::Vector3<f32>;
        let rotation: veclib::Quaternion<f32>;
        {
            // Set the variables since we can't have two mutable references at once
            rotation = entity.get_component::<components::Transform>(data.component_manager).unwrap().rotation;
            position = entity.get_component::<components::Transform>(data.component_manager).unwrap().position;
        }
        let camera_component = entity.get_component_mut::<components::Camera>(data.component_manager).unwrap();
        camera_component.update_projection_matrix(&data.custom_data.window);
        camera_component.update_view_matrix(position, rotation);
        camera_component.update_frustum_culling_matrix();
    }

    // Turn this into "Any" so we can cast into child systems
    fn as_any(&self) -> &dyn std::any::Any {
        self
    }

    fn as_any_mut(&mut self) -> &mut dyn std::any::Any {
        self
    }
}
