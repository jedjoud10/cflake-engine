use ecs::{Entity, FilteredLinkedComponents};
use systems::{System, SystemData, SystemEventType};
use world_data::WorldData;

use crate::components;

// Update the entities
pub fn entity_update(_system_data: &mut SystemData, _entity: &Entity, components: &FilteredLinkedComponents, data: &mut WorldData) {
    // Update the physics
    let transform = components.get_component_mut::<components::Transform>(data.component_manager).unwrap();
    let (mut position, mut rotation) = (transform.position, transform.rotation);
    let physics_object = components.get_component_mut::<components::Physics>(data.component_manager).unwrap();
    let physics_object = &mut physics_object.object;
    physics_object.update(&mut position, &mut rotation, data.time_manager.delta_time as f32);
    let transform = components.get_component_mut::<components::Transform>(data.component_manager).unwrap();
    transform.position = position;
    transform.rotation = rotation;
}

// Create a physics system
pub fn system(_world_data: &mut WorldData) -> System {
    let mut system = System::new();
    // Attach the events
    system.event(SystemEventType::EntityUpdate(entity_update));
    system
}
