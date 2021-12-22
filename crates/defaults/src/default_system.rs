use core::global::callbacks::CallbackType::*;
use others::callbacks::*;

// Some default events
pub fn entity_update(data: &mut (), entity: &ecs::Entity) {
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
        // Return the newly made system
        system
    });
}