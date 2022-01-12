use crate::{
    identifiers::{ComponentID, EntityID},
    ComponentError, ComponentLinkingGroup, EnclosedComponent, Entity, EntityError, System,
};
use ahash::AHashMap;
use bitfield::Bitfield;
use ordered_vec::shareable::ShareableOrderedVec;

// The Entity Component System manager that will handle everything ECS related
#[derive(Default)]
pub struct ECSManager {
    pub(crate) entities: ShareableOrderedVec<Entity>, // A vector full of entities. Each entity can get invalidated, but never deleted
    components: AHashMap<ComponentID, EnclosedComponent>, // The components that are valid in the world
    systems: Vec<System>,                             // Each system, stored in the order they were created
}
// Global code for the Entities, Components, and Systems
impl ECSManager {
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
    pub fn add_entity(&mut self, mut entity: Entity, id: EntityID, group: ComponentLinkingGroup) {
        // Check if the EntityID was not occupied already
        if self.entities.get(id.index as usize).is_some() {
            panic!()
        }
        entity.id = Some(id);
        // Add the entity
        let idx = self.entities.insert(id.index as usize, entity);
        // After doing that, we can safely add the components
        self.add_component_group(id, group).unwrap();
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
    fn add_component_group(&mut self, id: EntityID, group: ComponentLinkingGroup) -> Result<(), ComponentError> {
        for (cbitfield, boxed) in group.linked_components {
            self.add_component(id, boxed, cbitfield)?;
        }
        // Check if the linked entity is valid to be added into the systems
        self.systems.iter_mut().for_each(|system| system.check_add_entity(group.cbitfield, id));
        Ok(())
    }
    // Add a specific linked componment to the component manager. Return the said component's ID
    fn add_component(&mut self, id: EntityID, boxed: EnclosedComponent, cbitfield: Bitfield<u32>) -> Result<ComponentID, ComponentError> {
        // Create a new Component ID from an Entity ID
        let id = ComponentID::new(id, cbitfield);
        self.components.insert(id, boxed);
        Ok(id)
    }
    // Remove a specified component from the list
    fn remove_component(&mut self, id: ComponentID) -> Result<(), ComponentError> {
        // To remove a specific component just set it's component slot to None
        self.components
            .remove(&id)
            .ok_or(ComponentError::new("Tried removing component, but it was not present in the HashMap!".to_string(), id))?;
        Ok(())
    }
    /* #endregion */
    /* #region Systems */
    // Add a system to our current systems
    pub fn add_system(&mut self, system: System) {
        self.systems.push(system)
    }
    // Get a reference to the ecsmanager's systems.
    pub fn systems(&self) -> &[System] {
        self.systems.as_ref()
    }
    // Run the systems in sync, but their component updates is not
    // For now we will run them on the main thread, until I get my thread pool thingy working
    pub fn run_systems(&mut self) {
        // Filter the components for each system
        for system in self.systems.iter() {
            system.run_system(&mut self.components);
        }
    }
    /* #endregion */
}
