use ecs::{Entity, FilteredLinkedComponents};
use world_data::WorldData;
use systems::{InternalSystemData, System, SystemData, SystemEventType};

use crate::components;

// Update the entities
pub fn entity_update(system_data: &mut SystemData, entity: &Entity, components: &FilteredLinkedComponents, data: &mut WorldData) {
    // Update the physics
    let physics_object = components.get_component_mut::<components::Physics>(data.component_manager).unwrap();
    let physics_object = &mut physics_object.object;
    physics_object.update();
}

// Create a physics system
pub fn system(world_data: &mut WorldData) -> System {
    let mut system = System::new();
    // Attach the events
    system.event(SystemEventType::EntityUpdate(entity_update));
    system
}