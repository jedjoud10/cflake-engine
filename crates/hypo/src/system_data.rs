// Data that will be passed to the fire events in systems
pub struct SystemEventData<'a> {
    pub entity_manager: &'a mut hypo_ecs::EntityManager,
    pub component_manager: &'a mut hypo_ecs::ComponentManager,
    pub input_manager: &'a mut hypo_input::InputManager,
    pub shader_cacher: &'a mut (hypo_others::CacheManager<SubShader>, hypo_others::CacheManager<Shader>),
    pub texture_cacher: &'a mut hypo_others::CacheManager<Texture>,
    pub resource_manager: &'a mut ResourceManager,
    pub time_manager: &'a mut hypo_others::Time,
    pub debug: &'a mut debug::DebugRenderer,
    pub custom_data: &'a mut CustomWorldData,
}
// Data that will be passed some events in the systems that don't need all the world data
pub struct SystemEventDataLite<'a> {
    pub entity_manager: &'a mut EntityManager,
    pub component_manager: &'a mut ComponentManager,
    pub custom_data: &'a mut CustomWorldData,
}