use core::{global::callbacks::CallbackType::*, FrameID};
use ecs::SystemData;
use others::callbacks::*;
use rendering::GPUObjectID;

// Some default events
pub fn entity_update(data: &mut SystemData<()>, entity: &ecs::Entity) {
}

pub fn entity_removed(data: &mut SystemData<()>, entity: &ecs::Entity) {
}

// Create the default system
pub fn system() {
    core::global::ecs::add_system(|| {
        // Create a system
        let mut system = ecs::System::new(());
        // Link some components to the system
        system.link::<crate::components::Transform>();
        // And link the events
        system.event(ecs::SystemEventType::EntityUpdate(entity_update));
        system.event(ecs::SystemEventType::EntityRemoved(entity_removed));
        // Return the newly made system
        system
    });
}
