// Data that will be passed to the fire events in systems
pub struct WorldData<'a> {
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
    pub instance_manager: &'a mut others::InstanceManager,
}
// Some custom data that will be passed to systems
#[derive(Default)]
pub struct CustomWorldData {
    pub main_camera_entity_id: usize,
    pub sky_entity_id: usize,
    pub render_system_id: u8,
    pub window: rendering::Window,
}
