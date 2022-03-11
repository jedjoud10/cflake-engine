#[cfg(test)]
pub mod test {
    use crate::{
        component::{registry, Component, ComponentQueryParameters, ComponentQuerySet},
        entity::{ComponentLinkingGroup, ComponentUnlinkGroup, Entity},
        ECSManager,
    };
    use bitfield::Bitfield;
    use slotmap::Key;
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
    fn run_system(_world: &mut World, mut data: ComponentQuerySet) {
        for (_, components) in data[0].all.iter_mut() {
            let name = components.get_mut::<Name>().unwrap();
            *name = Name::new("Bob");
        }
    }

    // Run the systems in sync, but their component updates are not
    // Used only for testing
    fn run_systems<W>(ecs: &mut ECSManager<W>, world: &mut W) {
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

        let params = ComponentQueryParameters::default().link::<Name>();
        builder.query(params).event(run_system).build();

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
        fn run_internally(_world: &mut World, _data: ComponentQuerySet) {}
        let params = ComponentQueryParameters::default().link::<Name>();
        builder.query(params).event(run_internally).build();

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
        let params = ComponentQueryParameters::default().link::<Name>();
        builder.query(params).event(run_system).build();

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
    // Test multiple queries
    #[test]
    pub fn queries() {
        // Also create the context
        let mut world = 0;
        // Create the main ECS manager
        let mut ecs = ECSManager::<i32>::default();

        let builder = ecs.systems.builder();

        fn run_internally(_world: &mut i32, mut data: ComponentQuerySet) {
            let query1 = &data[0];
            let query2 = &data[1];
            let len1 = query1.all.len();
            let len2 = query2.all.len();
            if *_world == 0 {
                assert_eq!(len1, 1);
                assert_eq!(len2, 1);
                assert_eq!(query1.delta.added.len(), 1);
                assert_eq!(query2.delta.added.len(), 1);
            } else if *_world == 1 {
                assert_eq!(len1, 1);
                assert_eq!(len2, 1);
                assert_eq!(query1.delta.added.len(), 0);
                assert_eq!(query2.delta.added.len(), 0);
            } else if *_world == 2 {
                assert_eq!(len1, 0);
                assert_eq!(len2, 0);
                assert_eq!(query1.delta.removed.len(), 1);
                assert_eq!(query2.delta.removed.len(), 1);
                assert_eq!(query1.delta.added.len(), 0);
                assert_eq!(query2.delta.added.len(), 0);
            }
            *_world += 1;
        }

        // Query 1
        let params = ComponentQueryParameters::default().link::<Name>();
        // Query 2
        let params2 = ComponentQueryParameters::default().link::<Tagged>();
        builder.query(params).query(params2).event(run_internally).build();

        // Create a new entity
        let entity = Entity::default();
        let mut group = ComponentLinkingGroup::default();
        group.link::<Name>(Name::new("John")).unwrap();
        group.link::<Tagged>(Tagged::new("Person")).unwrap();
        let entity_key = ecs.add(entity, group).unwrap();
        assert!(!entity_key.is_null());
        let systems = ecs.systems.inner.borrow();
        let system = systems.get(0).unwrap();
        assert_eq!(system.subsystems.len(), 2);
        drop(systems);
        // Step 1
        run_systems(&mut ecs, &mut world);
        // Step 2 (systems actually store the entity's components)
        run_systems(&mut ecs, &mut world);
        ecs.remove(entity_key).unwrap();
        run_systems(&mut ecs, &mut world);
    }
}
