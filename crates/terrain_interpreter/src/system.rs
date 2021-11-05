use std::{collections::hash_map::DefaultHasher, hash::{Hash, Hasher}};

use crate::{Node, var_hash::{VarHash, VarHashType}};

// The main system that will be made from multiple densities and combiners
pub struct System {
    pub nodes: Vec<Box<dyn Node>>,
    pub vars: Vec<VarHash>,
}

impl Default for System {
    fn default() -> Self {
        // Create the default starter node
        Self { 
            nodes: Vec::new(),
            vars: Vec::new()
        }
    }
}

// Add nodes
impl System {
    // Add a specific node to the system
    pub fn add<T: Node>(&mut self, node: T) -> &VarHash {
        let id = self.vars.len();
        // Get the hash
        let hash = {
            let mut hash = DefaultHasher::new();
            id.hash(&mut hash);
            hash.finish()
        };
        // Create the var hash
        self.vars.push(VarHash { hash: hash, _type: VarHashType::Density });
        self.vars.get(id).unwrap()
    }
}