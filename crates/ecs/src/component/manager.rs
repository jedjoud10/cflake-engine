use ahash::AHashMap;
use bitfield::Bitfield;

use crate::{ComponentID, EnclosedComponent, EntityID, ComponentError};

// A component manager because we must store the components separately from the entities and systems
#[derive(Default)]
pub struct ComponentManager {    
    pub(crate) components: AHashMap<ComponentID, EnclosedComponent>, // The components that are valid in the world
}

impl ComponentManager {
    // Add a specific linked componment to the component manager. Return the said component's ID
    pub(crate) fn add_component(&mut self, id: EntityID, boxed: EnclosedComponent, cbitfield: Bitfield<u32>) -> Result<ComponentID, ComponentError> {
        // Create a new Component ID from an Entity ID
        let id = ComponentID::new(id, cbitfield);
        self.components.insert(id, boxed);
        Ok(id)
    }
    // Remove a specified component from the list
    pub(crate) fn remove_component(&mut self, id: ComponentID) -> Result<(), ComponentError> {
        // To remove a specific component just set it's component slot to None
        self.components
        .remove(&id)
        .ok_or(ComponentError::new("Tried removing component, but it was not present in the HashMap!".to_string(), id))?;
        Ok(())
    }
}