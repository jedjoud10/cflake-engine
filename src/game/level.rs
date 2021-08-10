use rand::prelude::SliceRandom;

use crate::engine::core::defaults::components::components::Camera;
use crate::engine::core::defaults::components::{components, *};
use crate::engine::core::defaults::systems::camera_system::CameraSystem;
use crate::engine::core::defaults::systems::sky_system::SkySystem;
use crate::engine::core::defaults::systems::*;
use crate::engine::core::ecs::entity::Entity;
use crate::engine::core::ecs::system::System;
use crate::engine::core::ecs::system_data::SystemEventData;
use crate::engine::core::world::World;
use crate::engine::rendering::model::Model;
use crate::engine::rendering::renderer::Renderer;
use crate::engine::rendering::shader::Shader;
use crate::engine::rendering::texture::Texture;
use crate::engine::rendering::*;
use rendering_system::RenderingSystem;

// Pre-register unused components
pub fn register_components(world: &mut World) {
	world
		.component_manager
		.register_component::<transforms::Position>();
	world
		.component_manager
		.register_component::<transforms::Rotation>();
}
// Load the systems
pub fn load_systems(world: &mut World) {
	// Load the default systems
	// Create the custom data
	let mut data: SystemEventData = SystemEventData {
		entity_manager: &mut world.entity_manager,
		component_manager: &mut world.component_manager,
		input_manager: &mut world.input_manager,
		shader_manager: &mut world.shader_manager,
		texture_manager: &mut world.texture_manager,
		time_manager: &mut world.time_manager,
		resource_manager: &mut world.resource_manager,
		custom_data: &mut world.custom_data,
	};

	// Load the rendering system
	let mut rendering_system = RenderingSystem::default();
	rendering_system.setup_system(&mut data);
	world.system_manager.add_system(rendering_system);

	// Load the camera system
	let mut camera_system = CameraSystem::default();
	camera_system.setup_system(&mut data);
	world.system_manager.add_system(camera_system);

	// Load the sky system
	let mut sky_system = SkySystem::default();
	sky_system.setup_system(&mut data);
	world.system_manager.add_system(sky_system);
}
// Load the entities
pub fn load_entities(world: &mut World) {
	// Create a camera entity
	let mut camera = Entity::new("Default Camera");
	camera.link_component::<transforms::Position>(
		&mut world.component_manager,
		transforms::Position {
			position: glam::vec3(5.0, 5.0, 5.0),
		},
	);
	camera.link_default_component::<transforms::Rotation>(&mut world.component_manager);
	camera.link_default_component::<components::Camera>(&mut world.component_manager);
	// Make it the default camera
	world.custom_data.main_camera_entity_id = world.add_entity(camera);

	// Load the default shader
	let default_shader_name: String = {
		let default_shader = Shader::from_vr_fr_subshader_files(
			"default.vrsh.glsl.pkg",
			"default.frsh.glsl.pkg",
			&mut world.resource_manager,
			&mut world.shader_manager,
		);
		default_shader.name.clone()
	};

	// Simple quad
	let mut quad = Entity::new("Quad");
	let shader_name: String = {
		let mut checkerboard_shader = Shader::from_vr_fr_subshader_files(
			"default.vrsh.glsl.pkg",
			"checkerboard.frsh.glsl.pkg",
			&mut world.resource_manager,
			&mut world.shader_manager,
		);
		checkerboard_shader.name.clone()
	};
	// Link the component
	let rc = Renderer::new(
		&mut world.resource_manager,
		&mut world.shader_manager,
		&shader_name,
		"quad.mdl3d.pkg",
	);
	quad.link_component::<Renderer>(&mut world.component_manager, rc);
	quad.link_default_component::<transforms::Position>(&mut world.component_manager);
	quad.link_component::<transforms::Rotation>(
		&mut world.component_manager,
		transforms::Rotation {
			rotation: glam::Quat::from_euler(glam::EulerRot::XYZ, -90.0_f32.to_radians(), 0.0, 0.0),
		},
	);
	quad.link_component::<transforms::Scale>(
		&mut world.component_manager,
		transforms::Scale { scale: 100.0 },
	);
	world.add_entity(quad);

	let mut cube = Entity::new("Cube");

	// Link the component
	let mut rc = Renderer::new_with_textures(
		&mut world.resource_manager,
		&mut world.texture_manager,
		&mut world.shader_manager,
		&default_shader_name,
		"cube.mdl3d.pkg",
		"diffuse.png.pkg",
		"normal.png.pkg",
	);
	rc.uv_scale *= 10.0;
	cube.link_component::<Renderer>(&mut world.component_manager, rc);
	cube.link_default_component::<transforms::Position>(&mut world.component_manager);
	cube.link_default_component::<transforms::Rotation>(&mut world.component_manager);
	cube.link_default_component::<transforms::Scale>(&mut world.component_manager);
	world.add_entity(cube);

	// Create the sky entity
	let mut sky = Entity::new("Sky");
	// Use a custom shader
	let sky_shader_name: String = {
		let mut shader = Shader::from_vr_fr_subshader_files(
			"default.vrsh.glsl.pkg",
			"sky.frsh.glsl.pkg",
			&mut world.resource_manager,
			&mut world.shader_manager,
		);
		shader.name.clone()
	};
	let mut rc = Renderer::new(
		&mut world.resource_manager,
		&mut world.shader_manager,
		&sky_shader_name,
		"sphere.mdl3d.pkg",
	);
	// Make the skysphere inside out, so we can see the insides only
	rc.model.flip_triangles();
	sky.link_component::<Renderer>(&mut world.component_manager, rc);
	sky.link_default_component::<transforms::Position>(&mut world.component_manager);
	sky.link_component::<transforms::Rotation>(
		&mut world.component_manager,
		transforms::Rotation {
			rotation: glam::Quat::from_euler(glam::EulerRot::XYZ, 90.0_f32.to_radians(), 0.0, 0.0),
		},
	);
	sky.link_component::<transforms::Scale>(
		&mut world.component_manager,
		transforms::Scale { scale: 900.0 },
	);
	sky.link_default_component::<components::Sky>(&mut world.component_manager);
	world.add_entity(sky);
}
