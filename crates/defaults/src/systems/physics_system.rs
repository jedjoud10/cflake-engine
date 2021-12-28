use core::global::callbacks::CallbackType::LocalEntityMut;
use ecs::SystemData;
use others::callbacks::{MutCallback, NullCallback};
// Update the entities
fn entity_update(data: &mut SystemData<()>, entity: &ecs::Entity) {
    // Update the physics
    core::global::ecs::entity_mut(
        entity.entity_id,
        LocalEntityMut(MutCallback::new(|entity| {
            // Get the transform position and rotation
            let transform = core::global::ecs::component_mut::<crate::components::Transform>(entity).unwrap();
            let (mut position, mut rotation) = (transform.position, transform.rotation);
            let physics = core::global::ecs::component_mut::<crate::components::Physics>(entity).unwrap();
            let physics_object = &mut physics.object;
            // Apply the physics step on the position and rotation
            physics_object.update(&mut position, &mut rotation, core::global::timings::delta() as f32);
            let transform = core::global::ecs::component_mut::<crate::components::Transform>(entity).unwrap();
            // Update the new position and rotation in the transform
            transform.position = position;
            transform.rotation = rotation;
        }))
        .create(),
    );
}

// Create the physics system
pub fn system() {
    core::global::ecs::add_system((), || {
        // Create a system
        let mut system = ecs::System::new();
        // Link some components to the system
        system.link::<crate::components::Transform>();
        system.link::<crate::components::Physics>();
        // And link the events
        system.event(ecs::SystemEventType::EntityUpdate(entity_update));
        // Return the newly made system
        system
    });
}
