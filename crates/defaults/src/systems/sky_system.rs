use super::super::components;
use ecs::{Entity, FilteredLinkedComponents};
use rendering::{Material, Model, Renderer, Shader, Texture2D, TextureWrapping};
use resources::LoadableResource;
use systems::{System, SystemData, SystemEventType};
use world_data::WorldData;

// Events
pub fn system_enabled(_system_data: &mut SystemData, data: &mut WorldData) {
    // Create the sky entity
    let mut sky = Entity::new("Sky");

    // Get shader name
    let sky_shader_name = Shader::new(
        vec!["defaults\\shaders\\rendering\\default.vrsh.glsl", "defaults\\shaders\\rendering\\sky.frsh.glsl"],
        &mut data.resource_manager,
        &mut data.shader_cacher,
        None,
        None
    )
    .1;

    // Load texture
    let cached_texture_id = Texture2D::new()
        .set_wrapping_mode(TextureWrapping::ClampToEdge)
        .load_texture("defaults\\textures\\sky_gradient.png", data.resource_manager, data.texture_cacher)
        .unwrap()
        .1;

    // Load model
    let mut model = Model::new().from_path("defaults\\models\\sphere.mdl3d", data.resource_manager).unwrap();
    model.flip_triangles();

    // Create a sky material
    let material = Material::default()
        .load_textures(&vec![Some(cached_texture_id), None], &mut data.texture_cacher)
        .set_shader(sky_shader_name.as_str());

    // Link components
    sky.link_component::<Renderer>(data.component_manager, Renderer::default().set_material(material).set_model(model).set_wireframe(false))
        .unwrap();

    sky.link_component::<components::Transform>(
        data.component_manager,
        components::Transform {
            scale: veclib::Vector3::ONE * 1900.0,
            ..components::Transform::default()
        },
    )
    .unwrap();

    sky.link_component::<components::Sky>(
        &mut data.component_manager,
        components::Sky {
            sky_gradient_texture_id: cached_texture_id,
        },
    )
    .unwrap();
    // Add entity
    data.custom_data.sky_entity_id = sky.entity_id;
    data.entity_manager.add_entity_s(sky);
}
fn entity_update(_system_data: &mut SystemData, _entity: &Entity, components: &FilteredLinkedComponents, data: &mut WorldData) {
    // Set the position of the sky sphere to always be the camera's position
    let position = data
        .entity_manager
        .get_entity(data.custom_data.main_camera_entity_id)
        .unwrap()
        .get_component::<components::Transform>(data.component_manager)
        .unwrap()
        .position;
    let transform = components.get_component_mut::<components::Transform>(data.component_manager).unwrap();
    // Update the position and update the matrix
    transform.position = position;
    transform.update_matrix();
}

// Create the sky system
pub fn system(data: &mut WorldData) -> System {
    let mut system = System::new();
    // Link the components
    system.link_component::<components::Sky>(data.component_manager).unwrap();
    system.link_component::<components::Transform>(data.component_manager).unwrap();
    // Attach the events
    system.event(SystemEventType::SystemEnabled(system_enabled));
    system.event(SystemEventType::EntityUpdate(entity_update));
    system
}
