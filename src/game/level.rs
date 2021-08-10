use rand::prelude::SliceRandom;

use crate::engine::core::defaults::components::components::Camera;
use crate::engine::core::defaults::components::{components, *};
use crate::engine::core::defaults::systems::*;
use crate::engine::core::defaults::systems::camera_system::CameraSystem;
use crate::engine::core::defaults::systems::sky_system::SkySystem;
use crate::engine::core::ecs::entity::Entity;
use crate::engine::core::ecs::system::System;
use crate::engine::core::ecs::system_data::FireData;
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
	let mut data: FireData = FireData {
		entity_manager: &mut world.entity_manager,
		component_manager: &mut world.component_manager,
		input_manager: &mut world.input_manager,
		shader_manager: &mut world.shader_manager,
		texture_manager: &mut world.texture_manager,
		time_manager: &mut world.time_manager,
		resource_manager: &mut world.resource_manager,
		custom_data: &mut world.custom_data
	};
	let mut rendering_system = RenderingSystem::default();
	rendering_system.setup_system(&mut data);
	world.system_manager.add_system(Box::new(rendering_system));

	let mut camera_system = CameraSystem::default();
	camera_system.setup_system(&mut data);
	world.system_manager.add_system(Box::new(camera_system));
	
	let mut sky_system = SkySystem::default();
	sky_system.setup_system(&mut data);
	world.system_manager.add_system(Box::new(sky_system));
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
    world.default_camera_id = world.add_entity(camera);

    // Load the default shader
    let mut default_shader_name: String;
    {
        let mut default_shader = Shader::from_vr_fr_subshader_files(
            "default.vrsh.glsl.pkg",
            "default.frsh.glsl.pkg",
            &mut world.resource_manager, &mut world.shader_manager
        );
        default_shader.finalize_shader();
        let default_shader = world.shader_manager.cache_shader(default_shader).unwrap();
        default_shader_name = default_shader.name.clone();
    }

    // Simple quad
    let mut quad = Entity::new("Quad");
    // Create the model
    let model = Model::from_resource(
        world
            .resource_manager
            .load_resource("quad.mdl3d.pkg", "models\\")
            .unwrap(),
    )
    .unwrap();
    // Link the component
    let mut rc = Renderer {
        model,
        shader_name: {
            let mut checkerboard_shader = Shader::from_vr_fr_subshader_files(
                "default.vrsh.glsl.pkg",
                "checkerboard.frsh.glsl.pkg",
                &mut world.resource_manager, &mut world.shader_manager
            );
            checkerboard_shader.finalize_shader();
            let checkerboard_shader = world
                .shader_manager
                .cache_shader(checkerboard_shader)
                .unwrap();
            checkerboard_shader.name.clone()
        },
        ..Renderer::default()
    };
    rc.refresh_model();
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
    for bunny_x in 0..6 {
        for bunny_y in 0..1 {
            for bunny_z in 0..6 {
                // Load a bunny model
                let mut bunny = Entity::new("Bunny");
                // Create the model
                let model2 = Model::from_resource(
                    world
                        .resource_manager
                        .load_resource("cube.mdl3d.pkg", "models\\")
                        .unwrap(),
                )
                .unwrap();
                // Link the component
                let rc = Renderer {
                    model: model2,
                    diffuse_texture_id: Texture::load_from_file("cute_saber_pic.png.pkg", world)
                        .unwrap(),
                    normal_texture_id: Texture::load_from_file("normal.png.pkg", world).unwrap(),
                    shader_name: default_shader_name.clone(),
                    ..Renderer::default()
                };
                bunny.link_component::<Renderer>(&mut world.component_manager, rc);
                bunny.link_component::<transforms::Position>(
                    &mut world.component_manager,
                    transforms::Position {
                        position: glam::vec3(
                            2.0 * bunny_x as f32,
                            2.0 * bunny_y as f32,
                            2.0 * bunny_z as f32,
                        ),
                    },
                );
                bunny.link_default_component::<transforms::Rotation>(&mut world.component_manager);
                bunny.link_default_component::<transforms::Scale>(&mut world.component_manager);
                world.add_entity(bunny);
            }
        }
    }
	/*
    // Create the sky entity
    let mut sky = Entity::new("Sky");
    let mut sky_model = Model::from_resource(
		world
		.resource_manager
		.load_resource("sphere.mdl3d.pkg", "models\\")
		.unwrap(),
    )
    .unwrap();
    sky_model.flip_triangles();
    // Use a custom shader
    let mut sky_shader_name: String = {
		let mut shader = Shader::from_vr_fr_subshader_files("default.vrsh.glsl.pkg", "sky.frsh.glsl.pkg", &mut world.resource_manager, &mut world.shader_manager);
        shader.finalize_shader();
        let cached_shader = world.shader_manager.cache_shader(shader).unwrap();
        cached_shader.name.clone()
    };

    let rc = Renderer {
		model: sky_model,
        shader_name: sky_shader_name.clone(),
        diffuse_texture_id: Texture::load_from_file("skytexture.png.pkg", world).unwrap(),
        ..Renderer::default()
    };
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
	*/
}
