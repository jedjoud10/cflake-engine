use others::callbacks::{MutCallback, NullCallback};
use ecs::stored::*;
use core::global::callbacks::CallbackType::LocalEntityMut;
// Update the entities
pub fn entity_update(data: &mut (), entity: &ecs::Entity) {
    // Update the physics
    core::global::ecs::entity_mut(entity, LocalEntityMut(MutCallback::new(|entity| {
        // Get the transform position and rotation
        let transform = core::global::ecs::component_mut2::<crate::components::Transform>(entity).unwrap();
        let (position, rotation) = (&mut transform.position, &mut transform.rotation);
        let physics = core::global::ecs::component_mut2::<crate::components::Physics>(entity).unwrap();
        let physics_object = &mut physics.object;
        // Apply the physics step on the position and rotation
        //physics_object.update(&mut position, &mut rotation, core);
        let transform = core::global::ecs::component_mut2::<crate::components::Transform>(entity).unwrap();
        // Update the new position and rotation in the transform
        transform.position = *position;
        transform.rotation = *rotation;
    })).create()); 
}

// Create the physics system
pub fn system() {
    core::global::ecs::add_system(|| {
        // Create a system
        let mut system = ecs::System::new(());
        // Link some components to the system
        system.link::<crate::components::Transform>();
        system.link::<crate::components::Physics>();
        // And link the events
        system.event(ecs::SystemEventType::EntityUpdate(entity_update));
        // Return the newly made system
        system
    });
}