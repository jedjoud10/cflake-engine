use hypo_ecs::*;
use hypo_errors::ECSError;
// Some system data that is part of a system and wrapped around System trait getter functions
#[derive(Default)]
pub struct SystemData {
    pub c_bitfield: u16,
    pub system_id: u8,
    pub state: SystemState,
    pub stype: SystemType,
    pub entities: Vec<u16>,
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
