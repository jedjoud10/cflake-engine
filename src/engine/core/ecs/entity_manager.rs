use std::collections::HashMap;

use super::entity::Entity;

// An entity manager that handles entities
#[derive(Default)]
pub struct EntityManager {
    pub entities: HashMap<u16, Entity>,
	pub entitites_to_add: Vec<Entity>
}

impl EntityManager {
    // Add an entity to the entity manager
    pub fn internal_add_entity(&mut self, mut entity: Entity) -> u16 {
        entity.entity_id = self.entities.len() as u16;
        println!(
            "\x1b[32mAdd entity '{}' with entity ID: {} and cBitfield: {}\x1b[0m",
            entity.name, entity.entity_id, entity.c_bitfield
        );
        // Add the entity to the world
        let id = entity.entity_id;
        self.entities.insert(entity.entity_id, entity);
        return id;
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
		return id;
	}
    // Get a mutable reference to a stored entity
    pub fn get_entity_mut(&mut self, entity_id: u16) -> Result<&mut Entity, super::error::EntityError> {
        if self.entities.contains_key(&entity_id) {
			return Ok(self.entities.get_mut(&entity_id).unwrap());
		} else {
			return Err(super::error::EntityError::new(format!("Entity with ID '{}' does not exist in EntityManager!", entity_id).as_str()));
		}
    }
    // Get an entity using it's entity id
    pub fn get_entity(&self, entity_id: u16) -> Result<&Entity, super::error::EntityError> {
        if self.entities.contains_key(&entity_id) {
			return Ok(self.entities.get(&entity_id).unwrap());
		} else {
			return Err(super::error::EntityError::new(format!("Entity with ID '{}' does not exist in EntityManager!", entity_id).as_str()));
		}
    }
    // Removes an entity from the world
    pub fn remove_entity(&mut self, entity_id: u16) -> Result<Entity, super::error::EntityError> {
		if self.entities.contains_key(&entity_id) {
			let removed_entity = self
            .entities
            .remove(&entity_id)
            .expect("Entity does not exist, so it could not be removed!");
			println!(
				"\x1b[33mRemove entity '{}' with entity ID: {} and cBitfield: {}\x1b[0m",
				removed_entity.name, removed_entity.entity_id, removed_entity.c_bitfield
			);
			return Ok(removed_entity);
		} else {
			return Err(super::error::EntityError::new(format!("Entity with ID '{}' does not exist in EntityManager!", entity_id).as_str()));
		}
    }
}
