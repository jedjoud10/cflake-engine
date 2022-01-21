use ahash::AHashSet;
use bitfield::Bitfield;

use crate::{
    component::{registry, Component, ComponentQuery, LinkedComponents},
    entity::EntityID,
    ECSManager,
};
use super::{EventHandler, SystemExecutionData};

// A system that updates specific components in parallel
pub struct System {
    // Our Component Bitfield
    pub(crate) cbitfield: Bitfield<u32>,
    // Event indices
    pub(crate) run_event_idx: isize,
    // Events
    entities: AHashSet<EntityID>,
}

impl Default for System {
    fn default() -> Self {
        System {
            cbitfield: Bitfield::<u32>::default(),
            run_event_idx: -1,
            entities: AHashSet::new(),
        }
    }
}

// System code
impl System {
    // Check if an entity validates our cbitfield
    pub(crate) fn check_cbitfield(&self, cbitfield: Bitfield<u32>) -> bool {
        cbitfield.contains(&self.cbitfield)
    }
    // Add an entity
    pub(crate) fn add_entity(&mut self, id: EntityID) {
        self.entities.insert(id);
    }
    // Remove an entity (It's cbitfield became innadequate for our system or the entity was removed from the world)
    pub(crate) fn remove_entity(&mut self, id: EntityID) {
        if self.entities.contains(&id) {
            self.entities.remove(&id);
        }
    }
    // Create a SystemExecutionData that we can actually run at a later time
    pub fn run_system<Context>(&self, ecs_manager: &ECSManager<Context>) -> SystemExecutionData<Context> {
        // These components are filtered for us
        let components = &ecs_manager.components.lock().unwrap();
        // Create the component query
        let components = self
            .entities
            .iter()
            .map(|id| {
                let entity = ecs_manager.entity(id).unwrap();
                let linked_components = LinkedComponents::new(entity, components);
                linked_components
            })
            .collect::<Vec<_>>();
        SystemExecutionData {
            run_event: ecs_manager.event_handler.get_run_event(self.run_event_idx).cloned(),
            query: ComponentQuery {
                linked_components: components,
                thread_pool: ecs_manager.thread_pool.clone(),
            },
        }
    }
}
