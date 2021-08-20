use crate::engine::{
    core::{
        cacher::CacheManager,
        world::{CustomWorldData, Time},
    },
    input::InputManager,
    rendering::{
        shader::{Shader, SubShader},
        texture::Texture,
    },
    resources::ResourceManager,
};

use super::{component::{ComponentID, ComponentManager}, entity::{Entity, EntityManager}, error::ECSError, system::EntityPrePassFilter};

// Data that will be passed to the fire events in systems
pub struct SystemEventData<'a> {
    pub entity_manager: &'a mut EntityManager,
    pub component_manager: &'a mut ComponentManager,
    pub input_manager: &'a mut InputManager,
    pub shader_cacher: &'a mut (CacheManager<SubShader>, CacheManager<Shader>),
    pub texture_cacher: &'a mut CacheManager<Texture>,
    pub resource_manager: &'a mut ResourceManager,
    pub time_manager: &'a mut Time,
    pub custom_data: &'a mut CustomWorldData,
}
// Data that will be passed some events in the systems that don't need all the world data
pub struct SystemEventDataLite<'a> {
    pub entity_manager: &'a mut EntityManager,
    pub component_manager: &'a mut ComponentManager,
    pub custom_data: &'a mut CustomWorldData,
}

// Some system data that is part of a system and wrapped around System trait getter functions
#[derive(Default)]
pub struct SystemData {
    pub c_bitfield: u16,
    pub system_id: u8,
    pub state: SystemState,
    pub stype: SystemType,
    pub entities: Vec<u16>,
    pub eppf: Option<Box<dyn EntityPrePassFilter>>,
}

impl SystemData {
    // Add a component to this system's component bitfield id
    pub fn link_component<T: ComponentID>(&mut self, component_manager: &mut ComponentManager) -> Result<(), ECSError> {
        if component_manager.is_component_registered::<T>() {
            self.c_bitfield |= component_manager.get_component_id::<T>()?;
        } else {
            component_manager.register_component::<T>();
            self.c_bitfield |= component_manager.get_component_id::<T>()?;
        }
        Ok(())
    }
}

// Tells you the state of the system, and for how long it's been enabled/disabled
#[derive(Clone, Copy)]
pub enum SystemState {
    Enabled(f32),
    Disabled(f32),
}
// Default system state
impl Default for SystemState {
    fn default() -> Self {
        Self::Enabled(0.0)
    }
}

// All of the systems that are implement by default
#[derive(Clone, Copy)]
pub enum SystemType {
    // Main System Types: Used for scripting
    Update,
    Tick,
    Render,
}
// Default system type
impl Default for SystemType {
    fn default() -> Self {
        Self::Update
    }
}
