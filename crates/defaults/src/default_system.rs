use core::global::callbacks::CallbackType::*;
use others::callbacks::*;

// Some default events
pub fn entity_update(data: &mut (), entity: &ecs::Entity) {
    let x = core::global::ecs::component::<crate::components::Transform>(entity).unwrap();
    core::global::ecs::entity_mut(entity, LocalEntityMut(MutCallback::new(|entity| {
        let comp = core::global::ecs::component_mut::<crate::components::Transform>(entity).unwrap();
        comp.position -= veclib::Vector3::X;
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