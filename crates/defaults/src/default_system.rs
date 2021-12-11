// Some default events
pub fn update_entity(data: &mut (), entity: &ecs::Entity) {
    println!("Update the entity {}", entity);
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
        // Return the newly made system
        system
    });
}
