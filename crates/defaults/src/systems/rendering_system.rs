// An improved multithreaded rendering system
use ecs::{Entity, FilteredLinkedComponents};
use rendering::pipec;
use systems::{System, SystemData, SystemEventType};
use world_data::WorldData;

use crate::components;

// Add the renderer in the render pipeline renderer
fn add_entity(sd: &mut SystemData, entity: &Entity, data: &mut WorldData) {
    pipec::add_renderer();
}
// Remove the renderer from the pipeline renderer
fn remove_entity(sd: &mut SystemData, entity: &Entity, data: &mut WorldData) {

}
// Send the updated data from the entity to the render pipeline as commands
fn update_entity(sd: &mut SystemData, entity: &Entity, flc: &FilteredLinkedComponents, data: &mut WorldData) {

}

// Create a rendering system
pub fn system(data: &mut WorldData) -> System {
    let mut system = System::default();
    // Link the components
    system.link_component::<components::Renderer>(data.component_manager).unwrap();
    system.link_component::<components::Transform>(data.component_manager).unwrap();

    system.event(SystemEventType::EntityAdded(add_entity));
    system.event(SystemEventType::EntityUpdate(update_entity));
    system.event(SystemEventType::EntityRemoved(remove_entity));
    // Attach the events
    system
}
