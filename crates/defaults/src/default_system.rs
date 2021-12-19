use core::global::callbacks::{CallbackType::*};
use others::callbacks::*;

// Some default events
pub fn update_entity(data: &mut (), entity: &ecs::Entity) {}

pub fn system_prefire(data: &mut ()) {
    // Create the linking group
    let mut linkings = ecs::ComponentLinkingGroup::new();
    linkings.link_default::<crate::components::Transform>().unwrap();
    // Add the entity
    let result = core::global::ecs::entity_add(ecs::Entity::new("Test-Entity"), linkings);
    result.with_callback(EntityRefCallbacks(RefCallback::new(|x| { })).create());
    let shared_data = rendering::SharedData::new(rendering::Model::default());
    let result = rendering::pipec::task(rendering::RenderTask::ModelCreate(shared_data));
    result.with_callback(GPUObjectCallback(OwnedCallback::new(|x| { })).create());
}

// Create the default system
pub fn system() {
    core::global::ecs::add_system(|| {
        // Create a system
        let mut system = ecs::System::new(());
        // Link some components to the system
        system.link::<crate::components::Transform>();
        // And link the events
        system.event(ecs::SystemEventType::EntityUpdate(update_entity));
        system.event(ecs::SystemEventType::SystemPrefire(system_prefire));
        // Return the newly made system
        system
    });
}
