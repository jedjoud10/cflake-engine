use crate::engine::rendering::*;
use crate::engine::core::defaults::components::{components, *};
use crate::engine::core::defaults::systems::*;
use crate::engine::core::ecs::{SystemType, SystemData, SystemState, System, Entity};
use crate::engine::core::world::World;
use crate::gl;
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
}
// Load the entities
pub fn load_entities(world: &mut World) {	
	// Create a camera entity
	let mut camera= Entity::default();	
	camera.name = String::from("Default Camera");	
	camera.link_component::<transforms::Position>(world, transforms::Position {
		position: glam::vec3(5.0, 5.0, 5.0),
	});	
	camera.link_component::<transforms::Rotation>(world, transforms::Rotation::default());	
	camera.link_component::<components::Camera>(world, components::Camera::default());

	// Make it the default camera
	world.default_camera_id = world.add_entity(camera);
	
	// Load the default shader
	let mut default_shader = Shader::default();
	{
		{
			let default_frag_subshader_resource = world.resource_manager.load_resource(String::from("default.frsh.glsl.pkg"), String::from("shaders\\")).unwrap();
			// Link the vertex and fragment shaders
			let mut frag_subshader = SubShader::from_resource(default_frag_subshader_resource).unwrap();
			// Compile the subshader
			frag_subshader.compile_subshader();
			// Cache it, and link it
			let mut frag_subshader = world.shader_manager.cache_subshader(frag_subshader).unwrap();
			default_shader.link_subshader(&frag_subshader);
		}
		{
			let default_vert_subshader_resource = world.resource_manager.load_resource(String::from("default.vrsh.glsl.pkg"), String::from("shaders\\")).unwrap();
			// Link the vertex and fragment shaders
			let mut vert_subshader = SubShader::from_resource(default_vert_subshader_resource).unwrap();
			// Compile the subshader
			vert_subshader.compile_subshader();
			// Cache it, and link it
			let mut vert_subshader = world.shader_manager.cache_subshader(vert_subshader).unwrap();
			default_shader.link_subshader(&vert_subshader);
		}
	}	
	let default_shader_name = default_shader.name.clone();
	let mut default_shader = world.shader_manager.cache_shader(default_shader).unwrap();
	// Use it for the default rendering of everything
	default_shader.use_shader();

	// Simple cube to render
	let mut cube = Entity::default();
	cube.name = String::from("Cube");
	// Create the model
	let model = Model::from_resource(world.resource_manager.load_resource(String::from("bunny.obj.pkg"), String::from("models\\")).unwrap()).unwrap();
	// Link the component
	let rc = components::Render {
    	render_state: EntityRenderState::Visible,
    	gpu_data: ModelDataGPU::default(),
    	shader_name: default_shader_name.clone(),   
		model	
	};
	cube.link_component::<components::Render>(world, rc);
	cube.link_component::<transforms::Position>(world, transforms::Position::default());
	cube.link_component::<transforms::Rotation>(world, transforms::Rotation::default());
	world.add_entity(cube);
	
}
