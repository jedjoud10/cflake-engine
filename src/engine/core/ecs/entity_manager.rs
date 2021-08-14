use std::collections::HashMap;

use super::entity::Entity;

// An entity manager that handles entities
#[derive(Default)]
pub struct EntityManager {
    pub entities: HashMap<u16, Entity>,
}

impl EntityManager {
    // Add an entity to the world
    pub fn add_entity(&mut self, mut entity: Entity) -> u16 {
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
    // Get a mutable reference to a stored entity
    pub fn get_entity_mut(&mut self, entity_id: u16) -> &mut Entity {
        self.entities.get_mut(&entity_id).unwrap()
    }
    // Get an entity using the entities vector and the "mapper (WIP)"
    pub fn get_entity(&self, entity_id: u16) -> &Entity {
        self.entities.get(&entity_id).unwrap()
    }
    // Removes an entity from the world
    pub fn remove_entity(&mut self, entity_id: u16) -> Entity {
        //println!("{:?}", self.entities);
        let removed_entity = self
            .entities
            .remove(&entity_id)
            .expect("Entity does not exist, so it could not be removed!");
        println!(
            "\x1b[33mRemove entity '{}' with entity ID: {} and cBitfield: {}\x1b[0m",
            removed_entity.name, removed_entity.entity_id, removed_entity.c_bitfield
        );
        return removed_entity;
    }
}
