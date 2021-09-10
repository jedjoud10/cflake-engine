use super::super::components;
use hypo_ecs::{Entity, FilteredLinkedComponents};
use hypo_input::*;
use hypo_system_event_data::{SystemEventData, SystemEventDataLite};
use hypo_systems::{System, SystemData};
#[derive(Default)]
pub struct UISystem {
    pub system_data: SystemData,
}

// The UI system which is going to render the elements and handle UI input for the elements
impl System for UISystem {
    // Wrappers around system data
    fn get_system_data(&self) -> &SystemData {
        &self.system_data
    }

    fn get_system_data_mut(&mut self) -> &mut SystemData {
        &mut self.system_data
    }

    // Setup the system
    fn setup_system(&mut self, data: &mut SystemEventData) {
        let system_data = self.get_system_data_mut(); 
    }

    // Called for each entity in the system
    fn fire_entity(&mut self, components: &FilteredLinkedComponents, data: &mut SystemEventData) {        
    }

    // Render all the elements onto the screen
    fn post_fire(&mut self, _data: &mut SystemEventData) {
        // Draw each element, from back to front
    }

    // Turn this into "Any" so we can cast into child systems
    fn as_any(&self) -> &dyn std::any::Any {
        self
    }

    fn as_any_mut(&mut self) -> &mut dyn std::any::Any {
        self
    }
}
