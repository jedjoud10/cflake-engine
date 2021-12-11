#[derive(Default)]
pub struct DefaultSystemData {}
impl ecs::CustomSystemData for DefaultSystemData {}


// Some default events
pub fn update_entity(data: &mut DefaultSystemData, entity: &ecs::Entity) {
    println!("Update the entity {}", entity);
}

// Launch the default system
pub fn system() -> ecs::System<DefaultSystemData> {
    // Create a system
    let mut system = ecs::System::<DefaultSystemData>::new(DefaultSystemData::default());
    // Link some components to the system
    system.link::<crate::components::Transform>();
    // And link the events
    system.event(ecs::SystemEventType::EntityUpdate(update_entity));
    // Return the newly made system
    system
}