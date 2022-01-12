use crate::{
    identifiers::EntityID,
    ComponentError, ComponentLinkingGroup, Entity, EntityError, System, ComponentManager,
};
use ordered_vec::shareable::ShareableOrderedVec;

// The Entity Component System manager that will handle everything ECS related (other than the components)
pub struct ECSManager<C> {
    pub(crate) entities: ShareableOrderedVec<Entity>, // A vector full of entities. Each entity can get invalidated, but never deleted
    systems: Vec<System<C>>,                             // Each system, stored in the order they were created
}

impl<C> Default for ECSManager<C> {
    fn default() -> Self {
        Self { entities: Default::default(), systems: Default::default() }
    }
}


// Global code for the Entities, Components, and Systems
impl<C> ECSManager<C> {
    /* #region Entities */
    // Get an entity
    pub fn entity(&self, id: &EntityID) -> Result<&Entity, EntityError> {
        self.entities.get(id.index as usize).ok_or(EntityError::new("Could not find entity!".to_string(), *id))
    }
    // Get an entity mutably
    pub fn entity_mut(&mut self, id: &EntityID) -> Result<&mut Entity, EntityError> {
        self.entities.get_mut(id.index as usize).ok_or(EntityError::new("Could not find entity!".to_string(), *id))
    }
    // Add an entity to the manager, and automatically link it's components
    pub fn add_entity(&mut self, mut entity: Entity, id: EntityID, group: ComponentLinkingGroup, component_manager: &mut ComponentManager) {
        // Check if the EntityID was not occupied already
        if self.entities.get(id.index as usize).is_some() {
            panic!()
        }
        entity.id = Some(id);
        // Add the entity
        let idx = self.entities.insert(id.index as usize, entity);
        // After doing that, we can safely add the components
        self.add_component_group(id, group, component_manager).unwrap();
    }
    // Remove an entity from the manager, and return it's value
    pub fn remove_entity(&mut self, id: EntityID) -> Result<Entity, EntityError> {
        // Invalidate the entity
        let res = self.entities.remove(id.index as usize).ok_or(EntityError::new("Could not find entity!".to_string(), id));
        res
    }
    /* #endregion */
    /* #region Components */
    // Add a component linking group to the manager
    fn add_component_group(&mut self, id: EntityID, group: ComponentLinkingGroup, component_manager: &mut ComponentManager) -> Result<(), ComponentError> {
        for (cbitfield, boxed) in group.linked_components {
            component_manager.add_component(id, boxed, cbitfield)?;
        }
        // Check if the linked entity is valid to be added into the systems
        self.systems.iter_mut().for_each(|system| system.check_add_entity(group.cbitfield, id));
        Ok(())
    }    
    /* #endregion */
    /* #region Systems */
    // Add a system to our current systems
    pub fn add_system(&mut self, system: System<C>) {
        self.systems.push(system)
    }
    // Get a reference to the ecsmanager's systems.
    pub fn systems(&self) -> &[System<C>] {
        self.systems.as_ref()
    }
    // Run the systems in sync, but their component updates is not
    // For now we will run them on the main thread, until I get my thread pool thingy working
    pub fn run_systems(&self, context: &C, component_manager: &mut ComponentManager) {
        // Filter the components for each system
        for system in self.systems.iter() {
            system.run_system(context, &mut component_manager.components);
        }
    }
    /* #endregion */
}
