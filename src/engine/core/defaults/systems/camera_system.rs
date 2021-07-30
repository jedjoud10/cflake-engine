use glam::Vec4Swizzles;

use crate::engine::rendering::*;
use crate::engine::core::defaults::components::{components, *};
use crate::engine::core::ecs::{SystemType, SystemData, SystemState, System, Entity};
use crate::engine::core::world::World;
use crate::gl;

// Create the camera system
pub fn create_system(world: &mut World) {
	// Create the default camera system
	let mut cs = System::default();
	cs.system_data.name = String::from("Camera System");
	cs.system_data.link_component::<components::Camera>(world);
	cs.system_data.link_component::<transforms::Position>(world);
	cs.system_data.link_component::<transforms::Rotation>(world);

	cs.system_data.entity_added_event = |entity, world| {
		// First time we initialize the camera, setup the matrices
		let mut position: glam::Vec3;
		let mut rotation: glam::Quat;
		{
			// Set the variables since we can't have two mutable references at once
			rotation = entity.get_component::<transforms::Rotation>(world).rotation;
			position = entity.get_component::<transforms::Position>(world).position;
		}
		let mut camera_component = entity.get_component_mut::<components::Camera>(world);
		camera_component.update_projection_matrix();
		camera_component.update_view_matrix(position, rotation);
		world.input_manager.bind_key(glfw::Key::W, String::from("camera_forward"));
		world.input_manager.bind_key(glfw::Key::S, String::from("camera_backwards"));
		world.input_manager.bind_key(glfw::Key::D, String::from("camera_right"));
		world.input_manager.bind_key(glfw::Key::A, String::from("camera_left"));
		world.input_manager.bind_key(glfw::Key::Space, String::from("camera_up"));
		world.input_manager.bind_key(glfw::Key::LeftShift, String::from("camera_down"));
	};

	cs.system_data.entity_loop_event = |entity, world| {
		let mut position: glam::Vec3;
		let mut rotation: glam::Quat;
		{
			// Create some movement using user input
			{
				let delta_time = world.time_manager.delta_time as f32;
				let mut changed_rotation = entity.get_component_mut::<transforms::Rotation>(world).rotation.clone();

				// Rotate the camera around
				let mouse_pos = world.input_manager.get_accumulated_mouse_position();
				let sensitivity = 0.001_f32;
				changed_rotation = glam::Quat::from_euler(glam::EulerRot::YXZ, -mouse_pos.0 as f32 * sensitivity, -mouse_pos.1 as f32 * sensitivity, 0.0);
			
				// Keyboard input
				let forward_vector = glam::Mat4::from_quat(changed_rotation).mul_vec4(glam::vec4(0.0, 0.0, 1.0, 1.0)).xyz();
				let up_vector = glam::Mat4::from_quat(changed_rotation).mul_vec4(glam::vec4(0.0, 1.0, 0.0, 1.0)).xyz();
				let right_vector = glam::Mat4::from_quat(changed_rotation).mul_vec4(glam::vec4(1.0, 0.0, 0.0, 1.0)).xyz();
				let changed_position = &mut entity.get_component::<transforms::Position>(world).position.clone();
				if world.input_manager.map_held(String::from("camera_forward")).0 {
					*changed_position += -forward_vector * world.time_manager.delta_time as f32;
				} else if world.input_manager.map_held(String::from("camera_backwards")).0 {
					*changed_position += forward_vector * world.time_manager.delta_time as f32;
				}
				if world.input_manager.map_held(String::from("camera_right")).0 {
					*changed_position += right_vector * world.time_manager.delta_time as f32;
				} else if world.input_manager.map_held(String::from("camera_left")).0 {
					*changed_position += -right_vector * world.time_manager.delta_time as f32;
				}
				if world.input_manager.map_held(String::from("camera_up")).0 {
					*changed_position += up_vector * world.time_manager.delta_time as f32;
				} else if world.input_manager.map_held(String::from("camera_down")).0 {
					*changed_position += -up_vector * world.time_manager.delta_time as f32;
				}
				// Update the variables
				*entity.get_component_mut::<transforms::Position>(world).position = **changed_position;
				entity.get_component_mut::<transforms::Rotation>(world).rotation = changed_rotation;
				position = *changed_position;
				rotation = changed_rotation.clone();
			}
		}
		let mut camera_component = entity.get_component_mut::<components::Camera>(world);
		// Update the view matrix every time we make a change
		camera_component.update_view_matrix(position, rotation);
	};

	world.add_system(cs);
	
}