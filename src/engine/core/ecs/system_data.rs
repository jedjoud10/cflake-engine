use std::collections::HashMap;

use crate::engine::{
    core::world::{CustomWorldData, Time, World},
    input::InputManager,
    rendering::{shader::ShaderManager, texture::TextureManager},
    resources::ResourceManager,
};

use super::{
    component::{ComponentID, ComponentManager},
    entity::EntityManager,
};

// Data that will be passed to the fire events in systems
pub struct SystemEventData<'a> {
    pub entity_manager: &'a mut EntityManager,
    pub component_manager: &'a mut ComponentManager,
    pub input_manager: &'a mut InputManager,
    pub shader_manager: &'a mut ShaderManager,
    pub texture_manager: &'a mut TextureManager,
    pub resource_manager: &'a mut ResourceManager,
    pub time_manager: &'a mut Time,
    pub custom_data: &'a mut CustomWorldData,
}
// Data that will be passed some events in the systems that don't need all the world data
pub struct SystemEventDataLite<'a> {
    pub entity_manager: &'a mut EntityManager,
    pub component_manager: &'a mut ComponentManager,
}

// Some system data that is part of a system and wrapped around System trait getter functions
#[derive(Clone)]
pub struct SystemData {
    pub c_bitfield: u16,
    pub system_id: u8,
    pub state: SystemState,
    pub stype: SystemType,
    pub entities: Vec<u16>,
    pub system_components: HashMap<u16, u16>,
}

// Default for system data
impl Default for SystemData {
    fn default() -> Self {
        Self {
            c_bitfield: 0,
            system_id: 0,
            state: SystemState::Enabled(0.0),
            stype: SystemType::Update,
            entities: Vec::new(),
            system_components: HashMap::new(),
        }
    }
}

impl SystemData {
    // Add a component to this system's component bitfield id
    pub fn link_component<T: ComponentID>(&mut self, component_manager: &mut ComponentManager) {
        if component_manager.is_component_registered::<T>() {
            self.c_bitfield = self.c_bitfield | component_manager.get_component_id::<T>();
        } else {
            component_manager.register_component::<T>();
            self.c_bitfield = self.c_bitfield | component_manager.get_component_id::<T>();
        }
        println!(
            "Link component '{}' to system '{}', with ID: {}",
            T::get_component_name(),
            self.system_id,
            component_manager.get_component_id::<T>()
        );
    }
}

// Tells you the state of the system, and for how long it's been enabled/disabled
#[derive(Clone, Copy)]
pub enum SystemState {
    Enabled(f32),
    Disabled(f32),
}

// All of the systems that are implement by default
#[derive(Clone, Copy)]
pub enum SystemType {
    // Main System Types: Used for scripting
    Update,
    Tick,
    Render,

    // Additional Default System: Uses the main system types
    Physics,
    GUI,
    Terrain,
}
