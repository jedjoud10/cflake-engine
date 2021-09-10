use hypo_main::*;
fn main() {
    // Load up the engine
    hypo_main::start(load_systems, load_entities);
}
// Load the systems
pub fn load_systems(world: &mut World) {
    // Load the default systems
    // Create the custom data
    let mut data: SystemEventData = SystemEventData {
        entity_manager: &mut world.entity_manager,
        component_manager: &mut world.component_manager,
        ui_manager: &mut world.ui_manager,
        input_manager: &mut world.input_manager,
        shader_cacher: &mut world.shader_cacher,
        texture_cacher: &mut world.texture_cacher,
        resource_manager: &mut world.resource_manager,
        time_manager: &mut world.time_manager,
        debug: &mut world.debug,
        custom_data: &mut world.custom_data,
    };

    // Load the rendering system
    let mut rendering_system = systems::RenderingSystem::default();
    rendering_system.setup_system(&mut data);
    world.system_manager.add_system(rendering_system);

    // Load the camera system
    let mut camera_system = systems::CameraSystem::default();
    camera_system.setup_system(&mut data);
    world.system_manager.add_system(camera_system);

    // Load the sky system
    let mut sky_system = systems::SkySystem::default();
    sky_system.setup_system(&mut data);
    world.system_manager.add_system(sky_system);

    // Load the terrain generator system
    let mut terrain_generator = Terrain::default();
    terrain_generator.setup_system(&mut data);
    world.system_manager.add_system(terrain_generator);

    // Load the UI system
    let mut ui_system = systems::UISystem::default();
    ui_system.setup_system(&mut data);
    world.system_manager.add_system(ui_system);
}
// Load the entities
pub fn load_entities(world: &mut World) {
    // Create a camera entity
    let mut camera = Entity::new("Default Camera");
    camera
        .link_component::<components::Transform>(
            &mut world.component_manager,
            components::Transform {
                position: veclib::Vector3::<f32>::new(5.0, 5.0, 5.0),
                ..components::Transform::default()
            },
        )
        .unwrap();
    camera.link_default_component::<components::Camera>(&mut world.component_manager).unwrap();
    // Make it the default camera
    world.custom_data.main_camera_entity_id = world.entity_manager.add_entity_s(camera);

    let mut entity = Entity::new("Sphere");

    let texture_ids = vec![
        Texture2D::new()
            .enable_mipmaps()
            .load_texture("user\\textures\\sandstone_cracks_diff_4k.png", &mut world.resource_manager, &mut world.texture_cacher)
            .unwrap()
            .1,
            Texture2D::new()
            .enable_mipmaps()
            .load_texture("user\\textures\\sandstone_cracks_nor_gl_4k.png", &mut world.resource_manager, &mut world.texture_cacher)
            .unwrap()
            .1,
    ];

    // Create a sky material
    let material = Material::default().load_textures(texture_ids, &mut world.texture_cacher).set_shader(&world.shader_cacher.1.id_get_default_object(0).unwrap().name);

    // Link components
    entity.link_component::<Renderer>(
        &mut world.component_manager,
        Renderer::default()
            .load_model("defaults\\models\\sphere.mdl3d", &mut world.resource_manager)
            .set_material(material)
    )
    .unwrap();
    entity.link_default_component::<components::AABB>(&mut world.component_manager).unwrap();
    entity.link_component::<components::Transform>(&mut world.component_manager, components::Transform::default().with_scale(veclib::Vector3::ONE * 10.0)).unwrap();
    world.entity_manager.add_entity_s(entity);
}
