use crate::{ECSError, LoadState};

use super::{Component, ComponentID, ComponentManager};
use others::{Instance, SmartList};
use std::collections::{HashMap, HashSet};

// An entity manager that handles entities
#[derive(Default)]
pub struct EntityManager {
    pub entities: SmartList<Entity>,
}

impl EntityManager {    
}

// A simple entity in the world
#[derive(Clone, Default)]
pub struct Entity {
    pub name: String,
    pub entity_id: usize,
    pub linked_components: HashMap<usize, usize>,
    pub c_bitfield: usize,
    pub load_state: LoadState,
}

// ECS time bois
impl Entity {
    // Create a new entity with a name
    pub fn new(name: &str) -> Self {
        Self {
            name: name.to_string(),
            ..Self::default()
        }
    }    
}

// Each entity is instantiable
impl Instance for Entity {
    fn set_name(&mut self, string: String) {
        self.name = string;
    }

    fn get_name(&self) -> String {
        self.name.clone()
    }
}
