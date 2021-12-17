use core::global::callbacks::{CallbackType::EntityRefCallbacks, RefCallback};

// Some default events
pub fn update_entity(data: &mut (), entity: &ecs::Entity) {    
    println!("Update the entity {}", entity);
    // When we create the new entity, we have a callback for it
}

pub fn system_prefire(data: &mut ()) {
    let x = core::global::ecs::entity_add_empty(ecs::Entity::new("Caca"));
    x.with_callback(EntityRefCallbacks(RefCallback::new(|x| { println!("Created entity") })).create())
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
