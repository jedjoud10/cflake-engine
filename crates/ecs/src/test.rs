#[cfg(test)]
pub mod test {
    use crate::{defaults::Name, linked_components::LinkedComponents, ComponentLinkingGroup, ECSManager, Entity, EntityID, System};

    // Some test contexts
    pub struct RefContext;
    pub struct MutContext;

    fn update_components(_context: &RefContext, components: &mut LinkedComponents) {
        // Get the component immutably
        let component = components.component::<Name>().unwrap();
        let name = &component.name;
        println!("{} {:?}", name, components.entity_id);

        // Write to the name
        let mut component2 = components.component_mut::<Name>().unwrap();
        component2.name = "Not a Person".to_string();
    }

    #[test]
    // Simple test to test the ecs
    pub fn test() {
        // Create the main ECS manager, and the Component Manager
        let mut ecs = ECSManager::new(|_| {});
        // Also create the contextes
        let ref_context = RefContext;
        let mut_context = MutContext;

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
        ecs.add_entity(&mut_context, entity, id, group);
        // The ID is valid now
        assert!(ecs.entity(&id2).is_ok());
        // Run the system for two frames
        ecs.run_systems(&ref_context, &mut_context);
        ecs.run_systems(&ref_context, &mut_context);
        ecs.run_systems(&ref_context, &mut_context);
        ecs.run_systems(&ref_context, &mut_context);
        // Remove the entity and check if the corresponding ID's became invalid
        let id4 = id3.clone();
        ecs.remove_entity(&mut_context, id3).unwrap();
        assert!(ecs.entity(&id4).is_err());
    }
    #[test]
    // Test the parralelization
    pub fn test_parallel() {
        // Create the main ECS manager, and the Component Manager
        let mut ecs = ECSManager::new(|_| {});
        // Also create the contextes
        let ref_context = RefContext;
        let mut_context = MutContext;

        // Make a simple system
        let mut hello_system = System::new();
        hello_system.link::<Name>();
        hello_system.set_event(update_components);
        hello_system.enable_multithreading();
        ecs.add_system(hello_system);

        // Create a simple entity with that component
        for x in 0..128 {
            let mut group = ComponentLinkingGroup::new();
            group.link(Name::new("Person")).unwrap();
            let entity = Entity::new();
            let id = EntityID::new(&ecs);
            // The entity is not created yet, so it is null
            ecs.add_entity(&mut_context, entity, id, group);
        }
        // Run the system for two frames
        ecs.run_systems(&ref_context, &mut_context);
        ecs.run_systems(&ref_context, &mut_context);
    }
}
