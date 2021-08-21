use glam::Vec4Swizzles;

use crate::engine::core::{
    defaults::components::{components, transforms},
    ecs::{
        component::FilteredLinkedComponents,
        entity::Entity,
        system::System,
        system_data::{SystemData, SystemEventData, SystemEventDataLite},
    },
};

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
        system_data.link_component::<transforms::Position>(data.component_manager).unwrap();
        system_data.link_component::<transforms::Rotation>(data.component_manager).unwrap();

        data.input_manager.bind_key(glfw::Key::W, "camera_forward");
        data.input_manager.bind_key(glfw::Key::S, "camera_backwards");
        data.input_manager.bind_key(glfw::Key::D, "camera_right");
        data.input_manager.bind_key(glfw::Key::A, "camera_left");
        data.input_manager.bind_key(glfw::Key::Space, "camera_up");
        data.input_manager.bind_key(glfw::Key::LeftShift, "camera_down");
        data.input_manager.bind_key(glfw::Key::G, "speed_switch");
        data.input_manager.bind_key(glfw::Key::J, "toggle_frustum_matrix_update");
    }

    // Called for each entity in the system
    fn fire_entity(&mut self, components: &FilteredLinkedComponents, data: &mut SystemEventData) {
        let position: glam::Vec3;
        let rotation: glam::Quat;
        {
            // Create some movement using user input
            {
                let _delta_time = data.time_manager.delta_time as f32;
                let changed_rotation: glam::Quat;

                // Rotate the camera around
                let mouse_pos = data.input_manager.get_accumulated_mouse_position();
                let sensitivity = 0.001_f32;
                changed_rotation = glam::Quat::from_euler(glam::EulerRot::YXZ, -mouse_pos.0 as f32 * sensitivity, -mouse_pos.1 as f32 * sensitivity, 0.0);

                // Keyboard input
                let forward_vector = glam::Mat4::from_quat(changed_rotation).mul_vec4(glam::vec4(0.0, 0.0, 1.0, 1.0)).xyz();
                let up_vector = glam::Mat4::from_quat(changed_rotation).mul_vec4(glam::vec4(0.0, 1.0, 0.0, 1.0)).xyz();
                let right_vector = glam::Mat4::from_quat(changed_rotation).mul_vec4(glam::vec4(1.0, 0.0, 0.0, 1.0)).xyz();
                let mut changed_position = components.get_component_mut::<transforms::Position>(data.component_manager).unwrap().position;
                let delta = data.time_manager.delta_time as f32;
                // Default speed
                let speed = 1.0 + data.input_manager.get_accumulated_mouse_scroll() * 0.1;
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
                components.get_component_mut::<transforms::Position>(data.component_manager).unwrap().position = changed_position;
                components.get_component_mut::<transforms::Rotation>(data.component_manager).unwrap().rotation = changed_rotation;
                position = changed_position;
                rotation = changed_rotation;
            }
        }
        let camera_component = components.get_component_mut::<components::Camera>(data.component_manager).unwrap();
        // Update the view matrix every time we make a change
        camera_component.update_view_matrix(position, rotation);
        camera_component.update_projection_matrix(&data.custom_data.window);
        // Update the frustum culling matrix if we toggled it
        if data.input_manager.map_held("toggle_frustum_matrix_update").0 {
            camera_component.update_frustum_culling_matrix();
        }
    }

    // When an entity gets added to this system
    fn entity_added(&mut self, entity: &Entity, data: &mut SystemEventDataLite) {
        // First time we initialize the camera, setup the matrices
        let position: glam::Vec3;
        let rotation: glam::Quat;
        {
            // Set the variables since we can't have two mutable references at once
            rotation = entity.get_component::<transforms::Rotation>(data.component_manager).unwrap().rotation;
            position = entity.get_component::<transforms::Position>(data.component_manager).unwrap().position;
        }
        let camera_component = entity.get_component_mut::<components::Camera>(data.component_manager).unwrap();
        camera_component.update_projection_matrix(&data.custom_data.window);
        camera_component.update_view_matrix(position, rotation);

        // Calculate the frustum planes at setup
        //let mut frustum = math::frustum::Frustum::default();
        //frustum.calculate_planes(position, rotation, camera_component);
    }

    // Turn this into "Any" so we can cast into child systems
    fn as_any(&self) -> &dyn std::any::Any {
        self
    }

    fn as_any_mut(&mut self) -> &mut dyn std::any::Any {
        self
    }
}
