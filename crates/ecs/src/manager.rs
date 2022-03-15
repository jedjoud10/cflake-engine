use std::{
    cell::{Ref, RefCell},
    rc::Rc,
};

use crate::{
    component::ComponentSet,
    entity::{ComponentLinkingGroup, ComponentUnlinkGroup, Entity, EntityKey, EntitySet},
    system::{System, SystemSet, SystemSettings},
    utils::{ComponentLinkingError, ComponentUnlinkError, EntityError},
};

// The Entity Component System manager that will handle everything ECS related
pub struct ECSManager<World> {
    pub entities: EntitySet,
    pub components: ComponentSet,
    pub systems: SystemSet<World>,
}

impl<World> Default for ECSManager<World> {
    fn default() -> Self {
        Self {
            entities: Default::default(),
            components: Default::default(),
            systems: Default::default(),
        }
    }
}

// Global code for the Entities, Components, and Systems
impl<World> ECSManager<World> {
    // Create the proper execution settings for systems, and return them
    pub fn ready(&mut self, frame: u128) -> (Rc<RefCell<Vec<System<World>>>>, SystemSettings) {
        self.components.ready_for_frame(frame).unwrap();
        (
            self.systems.inner.clone(),
            SystemSettings {
                to_remove: self.components.to_remove.clone(),
            },
        )
    }
    // Execute a bunch of systems
    pub fn execute_systems(systems: Ref<Vec<System<World>>>, world: &mut World, settings: SystemSettings) {
        for system in systems.iter() {
            system.run_system(world, settings.clone());
        }
    }

    // Wrapper functions
    // Entity adding/removing
    pub fn add(&mut self, group: ComponentLinkingGroup) -> Result<EntityKey, EntityError> {
        let key = self.entities.add(Entity::default())?;
        // Then link
        self.components
            .link(key, &mut self.entities, &mut self.systems, group)
            .map_err(|error| EntityError::new(error.details, key))?;
        Ok(key)
    }
    pub fn remove(&mut self, key: EntityKey) -> Result<(), EntityError> {
        self.entities.remove(key, &mut self.components, &mut self.systems)
    }
    // Linking / unlinking
    pub fn link(&mut self, key: EntityKey, group: ComponentLinkingGroup) -> Result<(), ComponentLinkingError> {
        self.components.link(key, &mut self.entities, &mut self.systems, group)
    }
    pub fn unlink_components(&mut self, key: EntityKey, group: ComponentUnlinkGroup) -> Result<(), ComponentUnlinkError> {
        self.components.unlink(key, &mut self.entities, &mut self.systems, group)
    }
}
