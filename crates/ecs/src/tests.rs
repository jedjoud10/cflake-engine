#[cfg(test)]
pub mod test {
    use crate::{
        component::{
            defaults::{Name, Tagged},
            registry,
        },
        entity::{ComponentLinkingGroup, ComponentUnlinkGroup, Entity, EntityID},
        event::EventKey,
        ECSManager,
    };
    use bitfield::Bitfield;

    // A test world
    pub struct World;
    fn run_system(_world: &mut World, mut data: EventKey) {
        let query = data.as_query_mut().unwrap();
        for (_, components) in query.lock().iter_mut() {
            let mut name = components.get_component_mut::<Name>().unwrap();
            *name = Name::new("Bob");
        }
    }

    #[test]
    // Simple test to test the ecs
    pub fn test() {
        // Also create the context
        let mut world = World;
        // Create the main ECS manager
        let mut ecs = ECSManager::<World>::default();

        // Make a simple system
        let builder = ecs.create_system_builder();
        builder.link::<Name>().with_run_event(run_system).build();

        // Create a simple entity with that component
        let mut group = ComponentLinkingGroup::default();
        group.link(Name::new("Person")).unwrap();
        let entity = Entity::default();
        let id = EntityID::new(&ecs);
        let id2 = id;
        let id3 = id;
        // The entity is not created yet, so it is null
        ecs.add_entity(entity, id, group).unwrap();
        // The ID is valid now
        assert!(ecs.get_entity(&id2).is_ok());
        // Run the system for two frames
        ecs.run_systems(&mut world);
        // Remove the entity and check if the corresponding ID's became invalid
        let id4 = id3;
        ecs.remove_entity(id3).unwrap();
        ecs.finish_update();
        let should_not_be_the_same = EntityID::new(&ecs);
        dbg!(id4);
        dbg!(should_not_be_the_same);
        assert_ne!(should_not_be_the_same, id4);
        assert!(ecs.get_entity(&id4).is_err());
        ecs.run_systems(&mut world);
        ecs.finish_update();
    }
    #[test]
    // Multithreaded stress test
    pub fn multithreaded_test() {
        // Also create the context
        let mut world = World;
        // Create the main ECS manager
        let mut ecs = ECSManager::<World>::default();

        // Make a simple system
        let builder = ecs.create_system_builder();
        fn internal_run(_world: &mut World, _data: EventKey) {
            /*
            // Transform the _context to RefContext using some magic fuckery
            components.update_all_threaded(|components| {
                let mut name = components.component_mut::<Name>().unwrap();
                *name = Name::new("Bob");
            });
            */
        }
        builder.link::<Name>().with_run_event(internal_run).build();

        // Create 10k entities
        for _x in 0..10_000 {
            // Create a simple entity with that component
            let mut group = ComponentLinkingGroup::default();
            group.link(Name::new("Person")).unwrap();
            let entity = Entity::default();
            let id = EntityID::new(&ecs);
            // The entity is not created yet, so it is null
            ecs.add_entity(entity, id, group).unwrap();
        }
        for _x in 0..10 {
            let i = std::time::Instant::now();
            ecs.run_systems(&mut world);
            println!("Took {}µs to update", i.elapsed().as_micros())
        }
    }
    #[test]
    pub fn test_direct() {
        // Also create the context
        let mut world = World;
        // Create the main ECS manager
        let mut ecs = ECSManager::<World>::default();

        // Make a simple system
        let builder = ecs.create_system_builder();
        builder.link::<Name>().with_run_event(run_system).build();

        // Add a new entity and play with it's components
        let entity = Entity::default();
        let id = EntityID::new(&ecs);
        ecs.add_entity(entity, id, ComponentLinkingGroup::default()).unwrap();
        assert!(ecs.get_entity(&id).is_ok());
        assert_eq!(ecs.get_entity(&id).unwrap().cbitfield, Bitfield::<u32>::default());
        let mut group = ComponentLinkingGroup::default();
        group.link(Name::new("Person")).unwrap();
        group.link(Tagged::new("Some interesting tag")).unwrap();
        ecs.link_components(id, group).unwrap();
        assert_ne!(ecs.get_entity(&id).unwrap().cbitfield, Bitfield::<u32>::default());
        ecs.run_systems(&mut world);
        let mut group = ComponentUnlinkGroup::default();
        group.unlink::<Tagged>().unwrap();
        ecs.unlink_components(id, group).unwrap();
        assert_eq!(ecs.get_entity(&id).unwrap().cbitfield, registry::get_component_bitfield::<Name>());
    }
    #[test]
    pub fn test_events() {
        // Create the main ECS manager
        let mut ecs = ECSManager::<World>::default();
        // Also create the context
        let mut world = World;

        // Make a simple system
        fn internal_run(_world: &mut World, mut data: EventKey) {
            let query = data.as_query_mut().unwrap();
            for (_, components) in query.lock().iter_mut() {
                let mut name = components.get_component_mut::<Name>().unwrap();
                dbg!("Internal Run");
                assert_eq!(*name.name, "John".to_string());
                *name = Name::new("Bob");
            }
        }
        fn internal_remove_entity(_world: &mut World, mut data: EventKey) {
            let query = data.as_query_mut().unwrap();
            for (_, components) in query.lock().iter_mut() {
                let name = components.get_component_mut::<Name>().unwrap();
                dbg!("Internal Remove Entity Run");
                assert_eq!(*name.name, "Bob".to_string());
            }
        }
        fn internal_add_entity(_world: &mut World, mut data: EventKey) {
            let query = data.as_query_mut().unwrap();
            for (_, components) in query.lock().iter_mut() {
                let name = components.get_component_mut::<Name>().unwrap();
                dbg!("Internal Add Entity Run");
                assert_eq!(*name.name, "John".to_string());
            }
        }
        let builder = ecs.create_system_builder();
        builder
            .link::<Name>()
            .with_run_event(internal_run)
            .with_removed_entities_event(internal_remove_entity)
            .with_added_entities_event(internal_add_entity)
            .build();

        // Add a new entity and play with it's components
        let entity = Entity::default();
        let id = EntityID::new(&ecs);
        let mut group = ComponentLinkingGroup::default();
        group.link::<Name>(Name::new("John")).unwrap();
        ecs.add_entity(entity, id, group).unwrap();
        ecs.run_systems(&mut world);
        ecs.remove_entity(id).unwrap();
        ecs.run_systems(&mut world);
        ecs.finish_update();
        // After this execution, the dangling components should have been removed
        assert_eq!(ecs.count_components(), 0);
    }
}
