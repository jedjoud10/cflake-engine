use glam::Vec4Swizzles;
use crate::engine::core::defaults::components::{components, *};
use crate::engine::core::ecs::{SystemType, System, SystemState, Entity};
use crate::engine::core::world::World;

// Create the camera system
pub fn create_system(world: &mut World) {
	// Default camera system
	let mut cs = System::default();
	cs.name = String::from("Camera System");
	cs.link_component::<components::Camera>(world);
	cs.link_component::<transforms::Position>(world);
	cs.link_component::<transforms::Rotation>(world);

	cs.entity_added_event = |entity, world, _| {
		// First time we initialize the camera, setup the matrices
		let position: glam::Vec3;
		let rotation: glam::Quat;
		{
			// Set the variables since we can't have two mutable references at once
			rotation = entity.get_component::<transforms::Rotation>(world).rotation;
			position = entity.get_component::<transforms::Position>(world).position;
		}
		let camera_component = entity.get_component_mut::<components::Camera>(world);
		camera_component.update_projection_matrix();
		camera_component.update_view_matrix(position, rotation);
		world.input_manager.bind_key(glfw::Key::W, "camera_forward");
		world.input_manager.bind_key(glfw::Key::S, "camera_backwards");
		world.input_manager.bind_key(glfw::Key::D, "camera_right");
		world.input_manager.bind_key(glfw::Key::A, "camera_left");
		world.input_manager.bind_key(glfw::Key::Space, "camera_up");
		world.input_manager.bind_key(glfw::Key::LeftShift, "camera_down");
		world.input_manager.bind_key(glfw::Key::G, "zoom");
		world.input_manager.bind_key(glfw::Key::H, "unzoom");
	};

	cs.entity_loop_event = |entity, world, _| {
		let position: glam::Vec3;
		let rotation: glam::Quat;
		let new_fov: f32;
		{
			// Create some movement using user input
			{
				let _delta_time = world.time_manager.delta_time as f32;
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
				let delta = world.time_manager.delta_time as f32;
				if world.input_manager.map_held("camera_forward").0 {
					*changed_position -= forward_vector * delta;
				} else if world.input_manager.map_held("camera_backwards").0 {
					*changed_position += forward_vector * delta;
				}
				if world.input_manager.map_held("camera_right").0 {
					*changed_position += right_vector * delta;
				} else if world.input_manager.map_held("camera_left").0 {
					*changed_position -= right_vector * delta;
				}
				if world.input_manager.map_held("camera_up").0 {
					*changed_position += up_vector * delta;
				} else if world.input_manager.map_held("camera_down").0 {
					*changed_position -= up_vector * delta;
				}
				let mut current_fov = entity.get_component_mut::<components::Camera>(world).horizontal_fov.clone();
				// Change the fov
				if world.input_manager.map_held("zoom").0 {
					current_fov += 5.0 * delta; 
				} else if world.input_manager.map_held("unzoom").0 {
					current_fov -= 5.0 * delta;
				}
				new_fov = current_fov;

				// Update the variables
				*entity.get_component_mut::<transforms::Position>(world).position = **changed_position;
				entity.get_component_mut::<transforms::Rotation>(world).rotation = changed_rotation;
				position = *changed_position;
				rotation = changed_rotation.clone();
			}
		}
		let camera_component = entity.get_component_mut::<components::Camera>(world);
		camera_component.horizontal_fov = new_fov;
		// Update the view matrix every time we make a change
		camera_component.update_view_matrix(position, rotation);
		camera_component.update_projection_matrix();
	};

	world.add_system(cs);
	
}