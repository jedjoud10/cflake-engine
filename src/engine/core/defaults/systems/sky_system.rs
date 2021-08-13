use crate::engine::core::defaults::components::{components, *};
use crate::engine::core::world::World;
use crate::engine::rendering::renderer::Renderer;
use crate::engine::rendering::shader::Shader;
use crate::engine::rendering::texture::TextureManager;
use glam::Vec4Swizzles;

use crate::engine::core::ecs::{
	entity::Entity,
	system::System,
	system_data::{SystemData, SystemEventData},
};

#[derive(Default)]
pub struct SkySystem {
	pub system_data: SystemData,
}

impl System for SkySystem {
	// Wrappers around system data
	fn get_system_data(&self) -> &SystemData {
		return &self.system_data;
	}

	fn get_system_data_mut(&mut self) -> &mut SystemData {
		return &mut self.system_data;
	}

	// Setup the system
	fn setup_system(&mut self, data: &mut SystemEventData) {
		let system_data = self.get_system_data_mut();
		system_data.link_component::<components::Sky>(data.component_manager);
		system_data.link_component::<transforms::Position>(data.component_manager);
		system_data.link_component::<transforms::Scale>(data.component_manager);
	}

	// Update the sun rotation
	fn pre_fire(&mut self, data: &mut SystemEventData) {
		data.custom_data.sun_rotation = glam::Quat::from_euler(glam::EulerRot::XYZ, data.time_manager.seconds_since_game_start as f32 / 4.0, data.time_manager.seconds_since_game_start as f32 / 4.0, data.time_manager.seconds_since_game_start as f32 / 4.0);
	}

	// Called for each entity in the system
	fn fire_entity(&mut self, entity: &mut Entity, data: &mut SystemEventData) {
		// Set the position of the sky sphere to always be the camera
		let position = data
			.entity_manager
			.get_entity(data.custom_data.main_camera_entity_id)
			.get_component::<transforms::Position>(data.component_manager)
			.position;
		*entity
			.get_component_mut::<transforms::Position>(data.component_manager)
			.position = *position;
	}

	// Add additional entities related to this system
	fn additional_entities(&mut self, data: &mut SystemEventData) -> Vec<Entity> {
		// Create the sky entity
		let mut sky = Entity::new("Sky");
		// Use a custom shader
		let sky_shader_name: String = {
			let mut shader = Shader::from_vr_fr_subshader_files(
				"shaders\\default.vrsh.glsl",
				"shaders\\sky.frsh.glsl",
				&mut data.resource_manager,
				&mut data.shader_manager,
			);
			shader.name.clone()
		};
		let mut rc = Renderer::new_with_textures(
			&mut data.resource_manager,
			&mut data.texture_manager,
			&mut data.shader_manager,
			&sky_shader_name,
			"models\\sphere.mdl3d",
			vec!["textures\\sky_gradient2.png"]
		);
		let id = data.texture_manager.get_texture_id("textures\\sky_gradient2.png");
		data.custom_data.sky_gradient_global_id = id;

		// Make the skysphere inside out, so we can see the insides only
		rc.model.flip_triangles();
		sky.link_component::<Renderer>(&mut data.component_manager, rc);
		sky.link_default_component::<transforms::Position>(&mut data.component_manager);
		sky.link_component::<transforms::Rotation>(
			&mut data.component_manager,
			transforms::Rotation {
				rotation: glam::Quat::from_euler(
					glam::EulerRot::XYZ,
					90.0_f32.to_radians(),
					0.0,
					0.0,
				),
			},
		);
		sky.link_component::<transforms::Scale>(
			&mut data.component_manager,
			transforms::Scale { scale: 900.0 },
		);
		sky.link_default_component::<components::Sky>(&mut data.component_manager);
		vec![sky]
	}

	// Turn this into "Any" so we can cast into child systems
	fn as_any(&self) -> &dyn std::any::Any {
		return self;
	}

	fn as_any_mut(&mut self) -> &mut dyn std::any::Any {
		return self;
	}
}
