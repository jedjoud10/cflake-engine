use glam::Vec4Swizzles;

use crate::engine::core::{
    defaults::components::{components, transforms},
    ecs::{
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
        return &self.system_data;
    }

    fn get_system_data_mut(&mut self) -> &mut SystemData {
        return &mut self.system_data;
    }

    // Setup the system
    fn setup_system(&mut self, data: &mut SystemEventData) {
        let mut system_data = self.get_system_data_mut();
        system_data.link_component::<components::Camera>(data.component_manager);
        system_data.link_component::<transforms::Position>(data.component_manager);
        system_data.link_component::<transforms::Rotation>(data.component_manager);

        data.input_manager.bind_key(glfw::Key::W, "camera_forward");
        data.input_manager
            .bind_key(glfw::Key::S, "camera_backwards");
        data.input_manager.bind_key(glfw::Key::D, "camera_right");
        data.input_manager.bind_key(glfw::Key::A, "camera_left");
        data.input_manager.bind_key(glfw::Key::Space, "camera_up");
        data.input_manager
            .bind_key(glfw::Key::LeftShift, "camera_down");
        data.input_manager.bind_key(glfw::Key::G, "zoom");
        data.input_manager.bind_key(glfw::Key::H, "unzoom");
    }

    // Called for each entity in the system
    fn fire_entity(&mut self, entity: &mut Entity, data: &mut SystemEventData) {
        let position: glam::Vec3;
        let rotation: glam::Quat;
        let new_fov: f32;
        {
            // Create some movement using user input
            {
                let _delta_time = data.time_manager.delta_time as f32;
                let mut changed_rotation = entity
                    .get_component_mut::<transforms::Rotation>(data.component_manager).unwrap()
                    .rotation
                    .clone();

                // Rotate the camera around
                let mouse_pos = data.input_manager.get_accumulated_mouse_position();
                let sensitivity = 0.001_f32;
                changed_rotation = glam::Quat::from_euler(
                    glam::EulerRot::YXZ,
                    -mouse_pos.0 as f32 * sensitivity,
                    -mouse_pos.1 as f32 * sensitivity,
                    0.0,
                );

                // Keyboard input
                let forward_vector = glam::Mat4::from_quat(changed_rotation)
                    .mul_vec4(glam::vec4(0.0, 0.0, 1.0, 1.0))
                    .xyz();
                let up_vector = glam::Mat4::from_quat(changed_rotation)
                    .mul_vec4(glam::vec4(0.0, 1.0, 0.0, 1.0))
                    .xyz();
                let right_vector = glam::Mat4::from_quat(changed_rotation)
                    .mul_vec4(glam::vec4(1.0, 0.0, 0.0, 1.0))
                    .xyz();
                let changed_position = &mut entity
                    .get_component::<transforms::Position>(data.component_manager).unwrap()
                    .position
                    .clone();
                let delta = data.time_manager.delta_time as f32;
                if data.input_manager.map_held("camera_forward").0 {
                    *changed_position -= forward_vector * delta;
                } else if data.input_manager.map_held("camera_backwards").0 {
                    *changed_position += forward_vector * delta;
                }
                if data.input_manager.map_held("camera_right").0 {
                    *changed_position += right_vector * delta;
                } else if data.input_manager.map_held("camera_left").0 {
                    *changed_position -= right_vector * delta;
                }
                if data.input_manager.map_held("camera_up").0 {
                    *changed_position += up_vector * delta;
                } else if data.input_manager.map_held("camera_down").0 {
                    *changed_position -= up_vector * delta;
                }
                let mut current_fov = entity
                    .get_component_mut::<components::Camera>(data.component_manager).unwrap()
                    .horizontal_fov
                    .clone();
                // Change the fov
                if data.input_manager.map_held("zoom").0 {
                    current_fov += 10.0 * delta;
                } else if data.input_manager.map_held("unzoom").0 {
                    current_fov -= 10.0 * delta;
                }
                new_fov = current_fov;

                // Update the variables
                *entity
                    .get_component_mut::<transforms::Position>(data.component_manager).unwrap()
                    .position = **changed_position;
                entity
                    .get_component_mut::<transforms::Rotation>(data.component_manager).unwrap()
                    .rotation = changed_rotation;
                position = *changed_position;
                rotation = changed_rotation.clone();
            }
        }
        let camera_component =
            entity.get_component_mut::<components::Camera>(data.component_manager).unwrap();
        camera_component.horizontal_fov = new_fov;
        // Update the view matrix every time we make a change
        camera_component.update_view_matrix(position, rotation);
        camera_component.update_projection_matrix();
    }

    // When an entity gets added to this system
    fn entity_added(&mut self, entity: &Entity, data: &mut SystemEventDataLite) {
        // First time we initialize the camera, setup the matrices
        let position: glam::Vec3;
        let rotation: glam::Quat;
        {
            // Set the variables since we can't have two mutable references at once
            rotation = entity
                .get_component::<transforms::Rotation>(data.component_manager).unwrap()
                .rotation;
            position = entity
                .get_component::<transforms::Position>(data.component_manager).unwrap()
                .position;
        }
        let camera_component =
            entity.get_component_mut::<components::Camera>(data.component_manager).unwrap();
        camera_component.update_projection_matrix();
        camera_component.update_view_matrix(position, rotation);
    }

    // Turn this into "Any" so we can cast into child systems
    fn as_any(&self) -> &dyn std::any::Any {
        return self;
    }

    fn as_any_mut(&mut self) -> &mut dyn std::any::Any {
        return self;
    }
}
