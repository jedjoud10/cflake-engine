#[cfg(test)]
pub mod test {
    use crate::{ECSManager, Entity, ComponentLinkingGroup, defaults::Name, System, SystemEventType, linked_components::LinkedComponents};

    fn update_components(c: &mut LinkedComponents) {
        let component = c.component::<Name>().unwrap();
        let name = &component.name; 
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
        let id = ecs.add_entity(Entity::new());
        ecs.add_component_group(id, group).unwrap();        

        // Run the system for one frame
        ecs.run_systems();
    }
}