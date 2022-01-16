#[cfg(test)]
pub mod test {
    use bitfield::Bitfield;

    use crate::{component::{ComponentQuery, Name, Tagged, registry}, ECSManager, entity::{ComponentLinkingGroup, Entity, EntityID, ComponentUnlinkGroup}};

    // A test context
    #[derive(Clone, Copy)]
    pub struct WorldContext;
    fn run_system(_context: WorldContext, components: ComponentQuery) {

        // Transform the _context to RefContext using some magic fuckery
        
        components.update_all(|components| {
            let mut i = 0;
            for x in 0..64 {
                i += x;
            }
            let name = components.component_mut::<Name>().unwrap();
        }, false);        
        
        /*
        let i = std::time::Instant::now();
        components.update_all(RefContext, |context, components| {
            let mut i = 0;
            for x in 0..1024 {
                i += x;
            }
        }, true);        
        println!("{}", i.elapsed().as_micros());
        */
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
        builder
            .link::<Name>()
            .set_event(run_system)
            .build();

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
        ecs.run_systems(context);
        // Remove the entity and check if the corresponding ID's became invalid
        let id4 = id3.clone();
        ecs.remove_entity(id3).unwrap();
        assert!(ecs.entity(&id4).is_err());
        ecs.run_systems(context);        
    }
    #[test]
    pub fn test_direct() {
        // Also create the context
        let context = WorldContext;
        // Create the main ECS manager
        let mut ecs = ECSManager::<WorldContext>::new(|| {});

        // Make a simple system
        let builder = ecs.create_system_builder();
        builder
            .link::<Name>()
            .set_event(run_system)
            .build();


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
}