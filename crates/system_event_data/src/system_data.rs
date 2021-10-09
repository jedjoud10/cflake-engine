// Data that will be passed to the fire events in systems
pub struct SystemEventData<'a> {
    pub entity_manager: &'a mut ecs::EntityManager,
    pub component_manager: &'a mut ecs::ComponentManager,
    pub ui_manager: &'a mut ui::UIManager,
    pub input_manager: &'a mut input::InputManager,
    pub shader_cacher: &'a mut (others::CacheManager<rendering::SubShader>, others::CacheManager<rendering::Shader>),
    pub texture_cacher: &'a mut others::CacheManager<rendering::Texture2D>,
    pub resource_manager: &'a mut resources::ResourceManager,
    pub time_manager: &'a mut others::Time,
    pub debug: &'a mut debug::MainDebug,
    pub custom_data: &'a mut CustomWorldData,
    pub instance_manager: &'a mut others::InstanceManager
}
// Data that will be passed some events in the systems that don't need all the world data
pub struct SystemEventDataLite<'a> {
    pub entity_manager: &'a mut ecs::EntityManager,
    pub component_manager: &'a mut ecs::ComponentManager,
    pub custom_data: &'a mut CustomWorldData,
}
// Some custom data that will be passed to systems
#[derive(Default)]
pub struct CustomWorldData {
    pub main_camera_entity_id: u16,
    pub sky_entity_id: u16,
    pub render_system_id: u8,
    pub window: rendering::Window,
}
