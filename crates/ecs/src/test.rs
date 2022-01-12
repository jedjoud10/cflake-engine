#[cfg(test)]
pub mod test {
    use crate::{defaults::Name, linked_components::LinkedComponents, ComponentLinkingGroup, ECSManager, Entity, EntityID, System, SystemEventType};

    fn update_components(c: &mut LinkedComponents) {
        // Get the component immutably
        let component = c.component::<Name>().unwrap();
        let name = &component.name;
        dbg!(name);

        // Write to the name
        let mut component2 = c.component_mut::<Name>().unwrap();
        component2.name = "Not a Person".to_string();
    }

    #[test]
    // Simple test to test the ecs
    pub fn test() {
        // Create the main ECS manager
        let mut ecs = ECSManager::default();
        // Make a simple system
        let mut hello_system = System::new();
        hello_system.link::<Name>();
        hello_system.event(SystemEventType::UpdateComponents(update_components));
        ecs.add_system(hello_system);

        // Create a simple entity with that component
        let mut group = ComponentLinkingGroup::new();
        group.link(Name::new("Person")).unwrap();
        let entity = Entity::new();
        let id = EntityID::new(&ecs);
        let id2 = id.clone();
        let id3 = id.clone();
        // The entity is not created yet, so it is null
        ecs.add_entity(entity, id, group);
        // The ID is valid now
        assert!(ecs.entity(&id2).is_ok());
        // Run the system for two frames
        ecs.run_systems();
        ecs.run_systems();
        // Remove the entity and check if the corresponding ID's became invalid
        let id4 = id3.clone();
        ecs.remove_entity(id3).unwrap();
        assert!(ecs.entity(&id4).is_err());
    }
}
