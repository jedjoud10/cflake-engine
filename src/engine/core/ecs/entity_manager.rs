use std::collections::HashMap;

use super::entity::Entity;
use super::error::ECSError;

// An entity manager that handles entities
#[derive(Default)]
pub struct EntityManager {
    pub entities: HashMap<u16, Entity>,
    pub entitites_to_add: Vec<Entity>,
}

impl EntityManager {
    // Add an entity to the entity manager
    pub fn internal_add_entity(&mut self, mut entity: Entity) -> u16 {
        entity.entity_id = self.entities.len() as u16;
        // Add the entity to the world
        let id = entity.entity_id;
        self.entities.insert(entity.entity_id, entity);
        id
    }
    // Add an entity to the entity manager temporarily, then call the actual add entity function on the world to actually add it
    pub fn add_entity_s(&mut self, mut entity: Entity) -> u16 {
        // Temporarily add it to the entities_to_add vector

        // Get the id of the entity inside the temp vector (Local ID)
        let mut id = self.entitites_to_add.len() as u16;
        // Add that id to the id of the current vector length (Global ID)
        id += self.entities.len() as u16;
        entity.entity_id = id;
        self.entitites_to_add.push(entity);
        id
    }
    // Get a mutable reference to a stored entity
    pub fn get_entity_mut(
        &mut self,
        entity_id: u16,
    ) -> Result<&mut Entity, ECSError> {
        if self.entities.contains_key(&entity_id) {
            return Ok(self.entities.get_mut(&entity_id).unwrap());
        } else {
            return Err(ECSError::new(
                format!(
                    "Entity with ID '{}' does not exist in EntityManager!",
                    entity_id
                )
                .as_str(),
            ));
        }
    }
    // Get an entity using it's entity id
    pub fn get_entity(&self, entity_id: u16) -> Result<&Entity, ECSError> {
        if self.entities.contains_key(&entity_id) {
            return Ok(self.entities.get(&entity_id).unwrap());
        } else {
            return Err(ECSError::new(
                format!(
                    "Entity with ID '{}' does not exist in EntityManager!",
                    entity_id
                )
                .as_str(),
            ));
        }
    }
    // Removes an entity from the world
    pub fn remove_entity(&mut self, entity_id: u16) -> Result<Entity, ECSError> {
        if self.entities.contains_key(&entity_id) {
            let removed_entity = self
                .entities
                .remove(&entity_id)
				.unwrap();
            Ok(removed_entity)
        } else {
            return Err(ECSError::new(
                format!(
                    "Entity with ID '{}' does not exist in EntityManager!",
                    entity_id
                )
                .as_str(),
            ));
        }
    }
}
