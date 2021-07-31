use crate::engine::rendering::*;
use crate::engine::core::defaults::components::{components, *};
use crate::engine::core::defaults::systems::*;
use crate::engine::core::ecs::{Entity};
use crate::engine::core::world::World;

// Pre-register unused components
pub fn register_components(world: &mut World) {
	world.component_manager.register_component::<transforms::Position>();
	world.component_manager.register_component::<transforms::Rotation>();
}
// Load the systems
pub fn load_systems(world: &mut World) {
	// Load the default systems
	// NOTE: The order of the systems actually matters
	camera_system::create_system(world);
	sky_system::create_system(world);
	rendering_system::create_system(world);	
}
// Load the entities
pub fn load_entities(world: &mut World) {	
	// Create a camera entity
	let mut camera= Entity::default();	
	camera.name = String::from("Default Camera");	
	camera.link_component::<transforms::Position>(world, transforms::Position {
		position: glam::vec3(5.0, 5.0, 5.0),
	});	
	camera.link_default_component::<transforms::Rotation>(world);	
	camera.link_default_component::<components::Camera>(world);

	// Make it the default camera
	world.default_camera_id = world.add_entity(camera);
	
	// Load the default shader
	let mut default_shader_name: String;
	{
		let mut default_shader = Shader::from_vr_fr_subshader_files("default.vrsh.glsl.pkg", "default.frsh.glsl.pkg", world);	
		let default_shader = world.shader_manager.cache_shader(default_shader).unwrap();
		default_shader_name = default_shader.name.clone();
	}
	
	// Simple quad
	let mut quad = Entity::default();
	quad.name = String::from("Quad");
	// Create the model
	let model = Model::from_resource(world.resource_manager.load_resource("quad.mdl3d.pkg", "models\\").unwrap()).unwrap();		
	// Link the component
	let mut rc = components::Renderer {
		model,
		shader_name: {
			let mut checkerboard_shader = Shader::from_vr_fr_subshader_files("default.vrsh.glsl.pkg", "checkerboard.frsh.glsl.pkg", world);	
			let checkerboard_shader = world.shader_manager.cache_shader(checkerboard_shader).unwrap();
			checkerboard_shader.name.clone()
		},
		..components::Renderer::default()
	};
	rc.refresh_model();
	quad.link_component::<components::Renderer>(world, rc);
	quad.link_default_component::<transforms::Position>(world);
	quad.link_component::<transforms::Rotation>(world, transforms::Rotation {
		rotation: glam::Quat::from_euler(glam::EulerRot::XYZ, -90.0_f32.to_radians(), 0.0, 0.0)
	});
	quad.link_component::<transforms::Scale>(world, transforms::Scale {
		scale: 100.0,
	});
	world.add_entity(quad);

	for bunny_index in 0..10 {
		// Load a bunny model
		let mut bunny = Entity::default();
		bunny.name = String::from("Bunny");
		// Create the model
		let model2 = Model::from_resource(world.resource_manager.load_resource("bunny.mdl3d.pkg", "models\\").unwrap()).unwrap();
		// Link the component
		let rc = components::Renderer {
			model: model2,
			shader_name: default_shader_name.clone(),
			..components::Renderer::default()
		};
		bunny.link_component::<components::Renderer>(world, rc);
		bunny.link_component::<transforms::Position>(world, transforms::Position {
			position: glam::vec3(10.0 + 5.0 * bunny_index as f32, 0.0, 0.0)
		});
		bunny.link_default_component::<transforms::Rotation>(world);
		bunny.link_default_component::<transforms::Scale>(world);
		world.add_entity(bunny);
	}

	
	// Create the sky entity
	let mut sky = Entity::default();
	sky.name = String::from("Sky");
	let mut sky_model = Model::from_resource(world.resource_manager.load_resource("sphere.mdl3d.pkg", "models\\").unwrap()).unwrap();
	sky_model.flip_triangles();
	// Use a custom shader
	let mut sky_shader_name: String = 
	{
		let mut shader = Shader::from_vr_fr_subshader_files("default.vrsh.glsl.pkg", "sky.frsh.glsl.pkg", world);	
		let cached_shader = world.shader_manager.cache_shader(shader).unwrap();
		cached_shader.name.clone()
	};

	let rc = components::Renderer {
		model: sky_model,
		shader_name: sky_shader_name.clone(),
		..components::Renderer::default()
	};	
	sky.link_component::<components::Renderer>(world, rc);
	sky.link_default_component::<transforms::Position>(world);
	sky.link_default_component::<transforms::Rotation>(world);
	sky.link_component::<transforms::Scale>(world, transforms::Scale {
		scale: 900.0
	});
	sky.link_default_component::<components::Sky>(world);
	world.add_entity(sky);	
}
