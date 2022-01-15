#[cfg(test)]
pub mod test {
    use crate::{defaults::Name, linked_components::{LinkedComponents, ComponentQuery}, ComponentLinkingGroup, ECSManager, Entity, EntityID, System, event_handler::EventHandler};

    // Some test contexts
    #[derive(Clone, Copy)]
    pub struct RefContext;
    pub struct MutContext;
    fn run_system(_context: RefContext, components: ComponentQuery) {

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
        // Also create the contextes
        let ref_context = RefContext;
        let mut mut_context = MutContext;
        // Create the main ECS manager, and the Event Handler
        let mut event_handler = EventHandler::<RefContext>::new();
        let mut ecs = ECSManager::new(|| {});

        // Make a simple system
        let mut hello_system = System::new();
        hello_system.link::<Name>();
        hello_system.set_event(&mut event_handler, run_system);
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
        ecs.run_systems(ref_context, &event_handler);
        // Remove the entity and check if the corresponding ID's became invalid
        let id4 = id3.clone();
        ecs.remove_entity(id3).unwrap();
        assert!(ecs.entity(&id4).is_err());
        ecs.run_systems(ref_context, &event_handler);
    }
    #[test]
    // Test the parralelization
    pub fn test_parallel() {
        // Create the main ECS manager, and the Component Manager
        let mut event_handler = EventHandler::<RefContext>::new();
        let mut ecs = ECSManager::new(|| {});
        // Also create the contextes
        let ref_context = RefContext;
        let mut mut_context = MutContext;

        // Make a simple system
        let mut hello_system = System::new();
        hello_system.link::<Name>();
        hello_system.set_event(&mut event_handler, run_system);
        ecs.add_system(hello_system);

        // Create a simple entity with that component
        for x in 0..4096 {
            let mut group = ComponentLinkingGroup::new();
            group.link(Name::new("Person")).unwrap();
            let entity = Entity::new();
            let id = EntityID::new(&ecs);
            // The entity is not created yet, so it is null
            ecs.add_entity(entity, id, group);
        }
        // Run the system for two frames    
        for x in 0..30 {
            let i = std::time::Instant::now();
            ecs.run_systems(ref_context, &event_handler);
            println!("{}", i.elapsed().as_millis());
        }    
        //ecs.run_systems(&mut mut_context);
    }
}