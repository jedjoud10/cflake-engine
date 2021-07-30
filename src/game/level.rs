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
	camera_system::create_system(world);
	rendering_system::create_system(world);	
	skysphere_system::create_system(world);
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

	// Simple cube to render
	let mut cube = Entity::default();
	cube.name = String::from("Cube");
	// Create the model
	let model = Model::from_resource(world.resource_manager.load_resource("cube.obj.pkg", "models\\").unwrap()).unwrap();
	// Link the component
	let rc = components::Render {
		model,
		shader_name: default_shader_name.clone(),
    	..components::Render::default()
	};
	cube.link_component::<components::Render>(world, rc);
	cube.link_default_component::<transforms::Position>(world);
	cube.link_default_component::<transforms::Rotation>(world);
	world.add_entity(cube);
	
	// Create the skysphere entity
	let mut skysphere = Entity::default();
	skysphere.name = String::from("Skysphere");
	let mut skysphere_model = Model::from_resource(world.resource_manager.load_resource("sphere.obj.pkg", "models\\").unwrap()).unwrap();
	skysphere_model.flip_triangles();
	let rc = components::Render {
		model: skysphere_model,
		shader_name: default_shader_name.clone(),
		..components::Render::default()
	};	
	skysphere.link_component::<components::Render>(world, rc);
	skysphere.link_default_component::<transforms::Position>(world);
	skysphere.link_default_component::<transforms::Rotation>(world);
	skysphere.link_default_component::<transforms::Scale>(world);
	skysphere.link_default_component::<components::Skysphere>(world);
	world.add_entity(skysphere);
}
