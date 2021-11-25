// An improved multithreaded rendering system
use ecs::{Entity, FilteredLinkedComponents};
use rendering::pipec;
use systems::{System, SystemData, SystemEventType};
use world_data::WorldData;

use crate::components;

// Add the renderer in the render pipeline renderer
fn add_entity(sd: &mut SystemData, entity: &Entity, data: &mut WorldData) {
    let renderer = entity.get_component::<components::Renderer>(data.component_manager).unwrap();
    let irenderer = renderer.internal_renderer.clone();
    let matrix = entity.get_component::<components::Transform>(data.component_manager).unwrap().matrix;
    let mut index = pipec::add_renderer(irenderer, matrix);
    // Update the index
    entity.get_component_mut::<components::Renderer>(data.component_manager).unwrap().internal_renderer.index = index;
}
// Remove the renderer from the pipeline renderer
fn remove_entity(sd: &mut SystemData, entity: &Entity, data: &mut WorldData) {
    let index = entity.get_component::<components::Renderer>(data.component_manager).unwrap().internal_renderer.clone().index;
    pipec::remove_renderer(index);
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
