#[cfg(test)]
pub mod test {
    use bitfield::Bitfield;

    use crate::{
        component::{
            defaults::{Name, Tagged},
            registry, ComponentQuery,
        },
        entity::{ComponentLinkingGroup, ComponentUnlinkGroup, Entity, EntityID},
        impl_component, ECSManager,
    };

    // A test context
    #[derive(Clone, Copy)]
    pub struct WorldContext;
    fn run_system(_context: WorldContext, components: ComponentQuery) {
        // Transform the _context to RefContext using some magic fuckery
        components.update_all(|components| {
            let mut name = components.component_mut::<Name>().unwrap();
            *name = Name::new("Bob");
        });
    }

    #[test]
    // Simple test to test the ecs
    pub fn test() {
        // Also create the context
        let context = WorldContext;
        // Create the main ECS manager
        let mut ecs = ECSManager::<WorldContext>::new(|| {});

        // Make a simple system
        let builder = ecs.create_system_builder();
        builder.link::<Name>().set_run_event(run_system).build();

        // Create a simple entity with that component
        let mut group = ComponentLinkingGroup::new();
        group.link(Name::new("Person")).unwrap();
        let entity = Entity::new();
        let id = EntityID::new(&ecs);
        let id2 = id;
        let id3 = id;
        // The entity is not created yet, so it is null
        ecs.add_entity(entity, id, group);
        // The ID is valid now
        assert!(ecs.entity(&id2).is_ok());
        // Run the system for two frames
        ecs.run_systems(context);
        // Remove the entity and check if the corresponding ID's became invalid
        let id4 = id3;
        ecs.remove_entity(id3).unwrap();
        ecs.finish_update();
        let should_not_be_the_same = EntityID::new(&ecs);
        dbg!(id4);
        dbg!(should_not_be_the_same);
        assert_ne!(should_not_be_the_same, id4);
        assert!(ecs.entity(&id4).is_err());
        ecs.run_systems(context);
        ecs.finish_update();
    }
    #[test]
    pub fn test_direct() {
        // Also create the context
        let context = WorldContext;
        // Create the main ECS manager
        let mut ecs = ECSManager::<WorldContext>::new(|| {});

        // Make a simple system
        let builder = ecs.create_system_builder();
        builder.link::<Name>().set_run_event(run_system).build();

        // Add a new entity and play with it's components
        let entity = Entity::new();
        let id = EntityID::new(&ecs);
        ecs.add_entity(entity, id, ComponentLinkingGroup::new());
        assert!(ecs.entity(&id).is_ok());
        assert_eq!(ecs.entity(&id).unwrap().cbitfield, Bitfield::<u32>::default());
        let mut group = ComponentLinkingGroup::new();
        group.link(Name::new("Person")).unwrap();
        group.link(Tagged::new("Some interesting tag")).unwrap();
        ecs.link_components(id, group).unwrap();
        assert_ne!(ecs.entity(&id).unwrap().cbitfield, Bitfield::<u32>::default());
        ecs.run_systems(context);
        let mut group = ComponentUnlinkGroup::new();
        group.unlink::<Tagged>().unwrap();
        ecs.unlink_components(id, group).unwrap();
        assert_eq!(ecs.entity(&id).unwrap().cbitfield, registry::get_component_bitfield::<Name>());
    }
    #[test]
    pub fn test_events() {
        // Create the main ECS manager
        let mut ecs = ECSManager::<WorldContext>::new(|| {});
        // Also create the context
        let context = WorldContext;

        // Make a simple system
        fn internal_run(_context: WorldContext, components: ComponentQuery) {
            components.update_all(|components| {
                let mut name = components.component_mut::<Name>().unwrap();
                dbg!("Internal Run");
                assert_eq!(*name.name, "John".to_string());
                *name = Name::new("Bob");
            });
        }
        fn internal_remove_entity(_context: WorldContext, components: ComponentQuery) {
            components.update_all(|components| {
                let name = components.component_mut::<Name>().unwrap();
                dbg!("Internal Remove Entity Run");
                assert_eq!(*name.name, "Bob".to_string());
            });
        }
        fn internal_add_entity(_context: WorldContext, components: ComponentQuery) {
            components.update_all(|components| {
                let name = components.component_mut::<Name>().unwrap();
                dbg!("Internal Add Entity Run");
                assert_eq!(*name.name, "John".to_string());
            });
        }
        let builder = ecs.create_system_builder();
        builder
            .link::<Name>()
            .set_run_event(internal_run)
            .set_removed_entities_event(internal_remove_entity)
            .set_added_entities_event(internal_add_entity)
            .build();

        // Add a new entity and play with it's components
        let entity = Entity::new();
        let id = EntityID::new(&ecs);
        let mut group = ComponentLinkingGroup::new();
        group.link::<Name>(Name::new("John")).unwrap();
        ecs.add_entity(entity, id, group);
        ecs.run_systems(context);
        ecs.remove_entity(id).unwrap();
        ecs.run_systems(context);
        ecs.finish_update();
        // After this execution, the dangling components should have been removed
        assert_eq!(ecs.count_components(), 0);
    }
    #[test]
    pub fn test_global_component() {
        // Also create the context
        let context = WorldContext;
        struct GlobalComponentTest {
            pub test_value: i32,
        }
        impl_component!(GlobalComponentTest);
        struct GlobalComponentTest2 {}
        impl_component!(GlobalComponentTest2);
        // Create the main ECS manager
        let mut ecs = ECSManager::<WorldContext>::new(|| {});
        // Make a simple system
        fn internal_run(_context: WorldContext, _query: ComponentQuery) {}

        assert!(ecs.global::<GlobalComponentTest>().is_ok());
        assert!(ecs.global::<GlobalComponentTest2>().is_err());
        let builder = ecs.create_system_builder();
        builder.link::<Name>().set_run_event(internal_run).build();
        ecs.run_systems(context);
    }
}
