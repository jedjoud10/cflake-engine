#[cfg(test)]
pub mod test {
    use crate::{
        component::{registry, Component},
        entity::{ComponentLinkingGroup, ComponentUnlinkGroup, Entity},
        event::EventKey,
        ECSManager,
    };
    use bitfield::Bitfield;
    // A name component that can be added to named entities
    #[derive(Component)]
    pub struct Name {
        pub name: String,
    }

    impl Default for Name {
        fn default() -> Self {
            Self { name: "Unnamed".to_string() }
        }
    }

    impl Name {
        pub fn new(name: &str) -> Self {
            Self { name: name.to_string() }
        }
    }

    // A tag component that can be added to entities that contain some sort of "Tag" We can then search for entities with the same tag
    #[derive(Component)]
    pub struct Tagged {
        pub tag: String,
    }

    impl Default for Tagged {
        fn default() -> Self {
            Self { tag: "Untagged".to_string() }
        }
    }

    impl Tagged {
        pub fn new(tag: &str) -> Self {
            Self { tag: tag.to_string() }
        }
    }

    // A test world
    pub struct World;
    fn run_system(_world: &mut World, mut data: EventKey) {
        let query = data.as_query_mut().unwrap();
        for (_, components) in query.iter_mut() {
            let mut name = components.get_mut::<Name>().unwrap();
            *name = Name::new("Bob");
        }
    }

    // Run the systems in sync, but their component updates are not
    // Used only for testing
    fn run_systems(ecs: &mut ECSManager<World>, world: &mut World) {
        ecs.components.clear_for_next_frame().unwrap();
        let (systems, settings) = ecs.ready();
        ECSManager::execute_systems(systems.borrow(), world, settings);
    }

    #[test]
    // Simple test to test the ecs
    pub fn test() {
        // Also create the context
        let mut world = World;
        // Create the main ECS manager
        let mut ecs = ECSManager::<World>::default();

        // Make a simple system
        let builder = ecs.systems.builder();
        builder.link::<Name>().with_run_event(run_system).build();

        // Create a simple entity with that component
        let mut group = ComponentLinkingGroup::default();
        group.link(Name::new("Person")).unwrap();
        let entity = Entity::default();
        // The entity is not created yet, so it is null
        let key2 = ecs.entities.add(entity).unwrap();
        ecs.components.link(key2, &mut ecs.entities, &mut ecs.systems, group).unwrap();
        let key3 = key2;
        // The ID is valid now
        assert!(ecs.entities.get(key2).is_ok());
        // Run the system for two frames
        run_systems(&mut ecs, &mut world);
        ecs.entities.remove(key3, &mut ecs.components, &mut ecs.systems).unwrap();
        run_systems(&mut ecs, &mut world);
    }
    #[test]
    // Multithreaded stress test
    pub fn multithreaded_test() {
        // Also create the context
        let mut world = World;
        // Create the main ECS manager
        let mut ecs = ECSManager::<World>::default();

        // Make a simple system
        let builder = ecs.systems.builder();
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
            // The entity is not created yet, so it is null
            let key = ecs.entities.add(entity).unwrap();
            ecs.components.link(key, &mut ecs.entities, &mut ecs.systems, group).unwrap();
        }
        for _x in 0..10 {
            let i = std::time::Instant::now();
            run_systems(&mut ecs, &mut world);
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
        let builder = ecs.systems.builder();
        builder.link::<Name>().with_run_event(run_system).build();

        // Add a new entity and play with it's components
        let entity = Entity::default();
        let key = ecs.entities.add(entity).unwrap();
        assert!(ecs.entities.get(key).is_ok());
        assert_eq!(ecs.entities.get(key).unwrap().cbitfield, Bitfield::<u32>::default());
        let mut group = ComponentLinkingGroup::default();
        group.link(Name::new("Person")).unwrap();
        group.link(Tagged::new("Some interesting tag")).unwrap();
        ecs.components.link(key, &mut ecs.entities, &mut ecs.systems, group).unwrap();
        assert_ne!(ecs.entities.get(key).unwrap().cbitfield, Bitfield::<u32>::default());
        run_systems(&mut ecs, &mut world);
        let mut group = ComponentUnlinkGroup::default();
        group.unlink::<Tagged>().unwrap();
        ecs.components.unlink(key, &mut ecs.entities, &mut ecs.systems, group).unwrap();
        assert_eq!(ecs.entities.get(key).unwrap().cbitfield, registry::get_component_bitfield::<Name>());
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
            for (_, components) in query.iter_mut() {
                let mut name = components.get_mut::<Name>().unwrap();
                dbg!("Internal Run");
                assert_eq!(*name.name, "John".to_string());
                *name = Name::new("Bob");
            }
        }
        fn internal_remove_entity(_world: &mut World, mut data: EventKey) {
            let query = data.as_query_mut().unwrap();
            for (_, components) in query.iter_mut() {
                let name = components.get_mut::<Name>().unwrap();
                dbg!("Internal Remove Entity Run");
                assert_eq!(*name.name, "Bob".to_string());
            }
        }
        fn internal_add_entity(_world: &mut World, mut data: EventKey) {
            let query = data.as_query_mut().unwrap();
            for (_, components) in query.iter_mut() {
                let name = components.get_mut::<Name>().unwrap();
                dbg!("Internal Add Entity Run");
                assert_eq!(*name.name, "John".to_string());
            }
        }
        ecs.systems
            .builder()
            .link::<Name>()
            .with_run_event(internal_run)
            .with_removed_entities_event(internal_remove_entity)
            .with_added_entities_event(internal_add_entity)
            .build();
        ecs.systems.builder().link::<Name>().link::<Tagged>().build();

        // Add a new entity and play with it's components
        let entity = Entity::default();
        let mut group = ComponentLinkingGroup::default();
        group.link::<Name>(Name::new("John")).unwrap();
        group.link(Tagged::new("Some interesting tag")).unwrap();
        let key = ecs.entities.add(entity).unwrap();
        ecs.components.link(key, &mut ecs.entities, &mut ecs.systems, group).unwrap();
        run_systems(&mut ecs, &mut world);
        ecs.entities.remove(key, &mut ecs.components, &mut ecs.systems).unwrap();
        run_systems(&mut ecs, &mut world);
        // After this execution, the dangling components should have been removed
        //assert_eq!(ecs.components.(), 0);
    }
}
