use core::{global::callbacks::CallbackType::*, FrameID};
use ecs::SystemData;
use others::callbacks::*;

pub struct SimpleSystem {
    pub c: u128
}
ecs::impl_systemdata!(SimpleSystem);

// Some default events
pub fn entity_update(data: &mut SystemData<SimpleSystem>, entity: &ecs::Entity) {
    core::global::ecs::entity_mut(entity.entity_id, LocalEntityMut(MutCallback::new(move |entity: &mut ecs::Entity| {
        let entity_id = entity.entity_id;
        let transform = core::global::ecs::component_mut::<crate::components::Transform>(entity).unwrap();
        if transform.position.x > 5.0 {
            core::global::ecs::entity_remove(entity_id);
        }
        transform.position += veclib::Vector3::X * 0.001;
        transform.update_frame_id = FrameID::now();
    })).create());
}


// Create the default system
pub fn system() {
    core::global::ecs::add_system(|| {
        // Create a system
        let mut system = ecs::System::new(SimpleSystem { c: 0 });
        // Link some components to the system
        system.link::<crate::components::Transform>();
        // And link the events
        system.event(ecs::SystemEventType::EntityUpdate(entity_update));
        // Return the newly made system
        system
    });
}
