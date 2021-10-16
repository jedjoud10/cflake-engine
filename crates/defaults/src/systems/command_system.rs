use system_event_data::WorldData;
use systems::{InternalSystemData, System, SystemData, SystemEventType};

use crate::components;

// Some custom system data
pub struct CustomData {
}

impl InternalSystemData for CustomData {
}

// System events
fn system_enabled(data: &mut SystemData, world_data: &mut WorldData) {
    println!("Command system enabled!");
}

pub fn system(world_data: &mut WorldData) -> System {
    let mut system = System::new();
    // Link the components
    system.link_component::<components::Sky>(world_data.component_manager).unwrap();
    // Attach the events
    system.event(SystemEventType::SystemEnabled(system_enabled));
    // Attach the custom system data
    system.custom_data(CustomData { });

    system
}