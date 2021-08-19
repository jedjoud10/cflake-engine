use crate::engine::core::defaults::components::{components, *};
use crate::engine::core::defaults::systems::camera_system::CameraSystem;
use crate::engine::core::defaults::systems::sky_system::SkySystem;
use crate::engine::core::defaults::systems::*;
use crate::engine::core::ecs::entity::Entity;
use crate::engine::core::ecs::system::System;
use crate::engine::core::ecs::system_data::SystemEventData;
use crate::engine::core::world::World;
use crate::engine::rendering::renderer::Renderer;
use crate::engine::rendering::shader::Shader;
use crate::engine::terrain::terrain::Terrain;
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
        shader_cacher: &mut world.shader_cacher,
        texture_cacher: &mut world.texture_cacher,
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

    // Load the terrain generator system
    let mut terrain_generator = Terrain::default();
    terrain_generator.setup_system(&mut data);
    world.system_manager.add_system(terrain_generator);
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
    ).unwrap();
    camera.link_default_component::<transforms::Rotation>(&mut world.component_manager).unwrap();
    camera.link_default_component::<components::Camera>(&mut world.component_manager).unwrap();
    // Make it the default camera
    world.custom_data.main_camera_entity_id = world.add_entity(camera);

    // Simple quad
    let mut quad = Entity::new("Quad");
    // Link the component
    let mut rc = Renderer::default();
    rc.load_model("models\\quad.mdl3d", &mut world.resource_manager);
    rc.shader_name = Shader::new(vec!["shaders\\default.vrsh.glsl", "shaders\\checkerboard.frsh.glsl"], &mut world.resource_manager, &mut world.shader_cacher).1;
    rc.load_default_textures(&mut world.texture_cacher);
    quad.link_component::<Renderer>(&mut world.component_manager, rc).unwrap();
    quad.link_default_component::<transforms::Position>(&mut world.component_manager).unwrap();
    quad.link_component::<transforms::Rotation>(
        &mut world.component_manager,
        transforms::Rotation {
            rotation: glam::Quat::from_euler(glam::EulerRot::XYZ, -90.0_f32.to_radians(), 0.0, 0.0),
        },
    ).unwrap();
    quad.link_component::<transforms::Scale>(
        &mut world.component_manager,
        transforms::Scale { scale: 100.0 },
    ).unwrap();
    world.add_entity(quad);

    // Anime moment
    let mut cube = Entity::new("Cube");
    // Link the component
    let mut rc = Renderer::default();
    rc.load_model("models\\cube.mdl3d", &mut world.resource_manager);
    rc.shader_name = world.shader_cacher.1.defaults[0].clone();
    rc.resource_load_textures(
        vec!["textures\\diffuse.png", "textures\\normals.png"],
        &mut world.texture_cacher,
        &mut world.resource_manager,
    );
    rc.uv_scale *= 10.0;
    cube.link_component::<Renderer>(&mut world.component_manager, rc).unwrap();
    cube.link_component::<transforms::Position>(&mut world.component_manager, transforms::Position {
		position: glam::vec3(0.0, 1.0, 0.0)
	}).unwrap();
    cube.link_default_component::<transforms::Rotation>(&mut world.component_manager).unwrap();
    cube.link_default_component::<transforms::Scale>(&mut world.component_manager).unwrap();
    world.add_entity(cube);
}
