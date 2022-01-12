#[cfg(test)]
pub mod test {
    use crate::{defaults::Name, linked_components::LinkedComponents, ComponentLinkingGroup, ECSManager, Entity, EntityID, System, ComponentManager};

    // A test world
    pub struct World();

    fn update_components(_context: &World, components: &mut LinkedComponents) {
        // Get the component immutably
        let component = components.component::<Name>().unwrap();
        let name = &component.name;
        dbg!(name);

        // Write to the name
        let mut component2 = components.component_mut::<Name>().unwrap();
        component2.name = "Not a Person".to_string();
    }

    #[test]
    // Simple test to test the ecs
    pub fn test() {
        // Create the main ECS manager, and the Component Manager
        let mut ecs = ECSManager::default();
        let mut cmanager = ComponentManager::default();
        // Also create the context
        let world = World();

        // Make a simple system
        let mut hello_system = System::new();
        hello_system.link::<Name>();
        hello_system.set_event(update_components);
        ecs.add_system(hello_system);

        // Create a simple entity with that component
        let mut group = ComponentLinkingGroup::new();
        group.link(Name::new("Person")).unwrap();
        let entity = Entity::new();
        let id = EntityID::new(&ecs);
        let id2 = id.clone();
        let id3 = id.clone();
        // The entity is not created yet, so it is null
        ecs.add_entity(entity, id, group, &mut cmanager);
        // The ID is valid now
        assert!(ecs.entity(&id2).is_ok());
        // Run the system for two frames
        ecs.run_systems(&world, &mut cmanager);
        ecs.run_systems(&world, &mut cmanager);
        // Remove the entity and check if the corresponding ID's became invalid
        let id4 = id3.clone();
        ecs.remove_entity(id3).unwrap();
        assert!(ecs.entity(&id4).is_err());
    }
}
