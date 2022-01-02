use crate::{
    identifiers::{ComponentID, IEntityID},
    ComponentError, Entity, EntityError, EnclosedComponent, System, ComponentLinkingGroup, EntityID,
};
use ahash::AHashMap;
use bitfield::Bitfield;
use ordered_vec::ordered_vec::OrderedVec;
use others::ExternalID;

// The Entity Component System manager that will handle everything ECS related
#[derive(Default)]
pub struct ECSManager {
    entities: OrderedVec<Entity>,                                        // A vector full of entities. Each entity can get invalidated, but never deleted
    components: AHashMap<ComponentID, EnclosedComponent>,                // The components that are valid in the world
    systems: Vec<System>,                                                // Each system, stored in the order they were created
    pub(crate) buffer: others::GlobalBuffer<EntityID, IEntityID>,        // A buffer that stores the actual internal value for the External Entity IDs
}
// Global code for the Entities, Components, and Systems
impl ECSManager {
    /* #region Entities */
    // Get an entity
    pub fn entity(&self, id: EntityID) -> Result<&Entity, EntityError> {
        let _id = *id.try_get(&self.buffer).ok_or(EntityError::new("The given entity ID is invalid!".to_string(), IEntityID::new(0)))?;
        self.entities.get(_id.index as usize).ok_or(EntityError::new("Could not find entity!".to_string(), _id))
    }
    // Get an entity mutably
    pub fn entity_mut(&mut self, id: EntityID) -> Result<&mut Entity, EntityError> {
        let _id = *id.try_get(&self.buffer).ok_or(EntityError::new("The given entity ID is invalid!".to_string(), IEntityID::new(0)))?;
        self.entities.get_mut(_id.index as usize).ok_or(EntityError::new("Could not find entity!".to_string(), _id))
    }
    // Add an entity to the manager, and automatically link it's components
    pub fn add_entity(&mut self, mut entity: Entity, group: ComponentLinkingGroup, external_id: EntityID) {
        // Create a new EntityID for this entity
        let entity_id = IEntityID::new(self.entities.get_next_idx() as u16);
        entity.id = entity_id;
        // Add the entity
        let idx = self.entities.push_shove(entity);
        let id_ref = &self.entities.get(idx).unwrap().id;
        // Update the given entity ID
        external_id.set(entity_id, &mut self.buffer); 
        // After doing that, we can safely add the components
        self.add_component_group(entity_id, group).unwrap();
    }
    // Remove an entity from the manager, and return it's value
    pub fn remove_entity(&mut self, external_id: EntityID) -> Result<Entity, EntityError> {
        // Invalidate an entity
        let _id = external_id.try_get(&mut self.buffer).ok_or(EntityError::new("The given entity ID is invalid!".to_string(), IEntityID::new(0)))?;
        let res = self.entities.remove(_id.index as usize).ok_or(EntityError::new("Could not find entity!".to_string(), *_id));
        // Since the entity got removed, we must invalidate the ptr stored in the AtomicPtr
        external_id.invalidate(&mut self.buffer);
        res
    }
    /* #endregion */
    /* #region Components */
    // Add a component linking group to the manager
    fn add_component_group(&mut self, id: IEntityID, group: ComponentLinkingGroup) -> Result<(), ComponentError> {
        for (cbitfield, boxed) in group.linked_components {
            self.add_component(id, boxed, cbitfield)?;
        }
        // Check if the linked entity is valid to be added into the systems
        self.systems.iter_mut().for_each(|system| system.check_add_entity(group.cbitfield, id));
        Ok(())
    }
    // Add a specific linked componment to the component manager. Return the said component's ID
    fn add_component(&mut self, id: IEntityID, boxed: EnclosedComponent, cbitfield: Bitfield<u32>) -> Result<ComponentID, ComponentError> {
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
