use crate::manager::EcsManager;
use super::Entity;

// And entity modifier that will be able to add/remove components directly from an entity
pub struct EntityModifier<'a> {
    // The entity ID
    entity: Entity,

    // The ecs manager
    manager: &'a mut EcsManager,
}

impl<'a> EntityModifier<'a> {
    // Create a new entity modifier
    pub fn new(entity: Entity, manager: &'a mut EcsManager) -> Self {
        Self {
            entity,
            manager,
        }
    }

    // Add a com
}