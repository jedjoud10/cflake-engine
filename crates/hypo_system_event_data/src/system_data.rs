// Data that will be passed to the fire events in systems
pub struct SystemEventData<'a> {
    pub entity_manager: &'a mut hypo_ecs::EntityManager,
    pub component_manager: &'a mut hypo_ecs::ComponentManager,
    pub input_manager: &'a mut hypo_input::InputManager,
    pub shader_cacher: &'a mut (hypo_others::CacheManager<hypo_rendering::SubShader>, hypo_others::CacheManager<hypo_rendering::Shader>),
    pub texture_cacher: &'a mut hypo_others::CacheManager<hypo_rendering::Texture>,
    pub resource_manager: &'a mut hypo_resources::ResourceManager,
    pub time_manager: &'a mut hypo_others::Time,
    pub debug: &'a mut hypo_debug::DebugRenderer,
    pub custom_data: &'a mut CustomWorldData,
}
// Data that will be passed some events in the systems that don't need all the world data
pub struct SystemEventDataLite<'a> {
    pub entity_manager: &'a mut hypo_ecs::EntityManager,
    pub component_manager: &'a mut hypo_ecs::ComponentManager,
    pub custom_data: &'a mut CustomWorldData,
}
// Some custom data that will be passed to systems
#[derive(Default)]
pub struct CustomWorldData {
    pub main_camera_entity_id: u16,
    pub sky_entity_id: u16,
    pub render_system_id: u8,
    pub window: hypo_rendering::Window,
}
