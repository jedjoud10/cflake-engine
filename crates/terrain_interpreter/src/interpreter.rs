use std::{collections::hash_map::DefaultHasher, hash::{Hash, Hasher}};

use crate::{Node, nodes::base_position::BasePosition, var_hash::{VarHash, VarHashType}};

// The main system that will be made from multiple densities and combiners
pub struct Interpreter {
    pub nodes: Vec<Node>,
    pub vars: Vec<VarHash>,
}

impl Default for Interpreter {
    fn default() -> Self {
        // Create the default starter node
        let mut default = Self { 
            nodes: Vec::new(),
            vars: Vec::new()
        };
        default.add(Node::new_base::<BasePosition>());
        default
    }
}

// Add nodes
impl Interpreter {
    // Add a specific node to the system
    pub fn add(&mut self, node: Node) -> &VarHash {
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