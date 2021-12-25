use core::{global::callbacks::CallbackType::*, FrameID};
use others::callbacks::*;

// Some default events
pub fn entity_update(data: &mut (), entity: &ecs::Entity) {
    core::global::ecs::entity_mut(entity.entity_id, LocalEntityMut(MutCallback::new(|entity| {
        let transform = core::global::ecs::component_mut::<crate::components::Transform>(entity).unwrap();
        transform.position += veclib::Vector3::X * 0.0001;
        transform.update_frame_id = FrameID::now();
    })).create())
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
