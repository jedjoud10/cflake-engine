#[cfg(test)]
pub mod test {
    use others::ExternalID;

    use crate::{ECSManager, Entity, ComponentLinkingGroup, defaults::Name, System, SystemEventType, linked_components::LinkedComponents, IEntityID, EntityID};

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
        let id = EntityID::new();
        let id2 = id.clone();
        let id3 = id.clone();
        // The entity is not created yet, so it is null
        assert!(id.try_get(&ecs.buffer).is_none());
        assert!(ecs.entity(id2.clone()).is_err());
        ecs.add_entity(entity, group, id);
        dbg!(id2.try_get(&ecs.buffer));
        dbg!(id3.try_get(&ecs.buffer));
        // The ID is valid now
        assert_eq!(id2.try_get(&ecs.buffer).unwrap(), id3.try_get(&ecs.buffer).unwrap());
        // Run the system for two frames
        ecs.run_systems();
        ecs.run_systems();
        // Remove the entity and check if the corresponding ID's became invalid
        let id4 = id2.clone();
        ecs.remove_entity(id2).unwrap();
        assert_eq!(id4.try_get(&ecs.buffer), None);
    }
}