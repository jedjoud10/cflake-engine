use std::{
    collections::hash_map::DefaultHasher,
    hash::{Hash, Hasher},
};

use math::bounds::AABB;

use crate::{
    error::InterpreterError,
    nodes::{base_position::BasePosition, final_density::FinalDensity},
    var_hash::{VarHash, VarHashType},
    Influence, NodeInterpreter,
};

// The main system that will be made from multiple densities and combiners
pub struct Interpreter {
    pub nodes: Vec<Box<dyn NodeInterpreter>>,
    pub vars: Vec<VarHash>,
    pub lines: Vec<String>,
    pub finalized: bool,
    pub max_influence: Influence,
}

impl Default for Interpreter {
    fn default() -> Self {
        // Create the default starter node
        let default = Self {
            nodes: Vec::new(),
            vars: Vec::new(),
            lines: Vec::new(),
            finalized: false,
            max_influence: Influence::Modified(-1.0, 1.0),
        };
        default
    }
}

// Add nodes
impl Interpreter {
    // Add a specific node to the system
    pub fn add<T: NodeInterpreter>(&mut self, node_interpreter: T, inputs: Vec<VarHash>) -> Result<VarHash, InterpreterError> {
        let id = self.vars.len();
        // Create the var hash
        let boxed = Box::new(node_interpreter);
        let var_hash = VarHash {
            name: id,
            _type: boxed.get_output_type(),
        };
        self.vars.push(var_hash.clone());
        // Create a variable for this node
        let line = format!(
            "{} {} = {};",
            var_hash._type.to_glsl_type(),
            boxed.custom_name(var_hash.get_name()),
            boxed.get_node_string(&inputs)?
        );
        self.lines.push(line);
        Ok((*self.vars.get(id).unwrap()).clone())
    }
    // Finalize the tree with a specific var hash
    pub fn finalize(&mut self, final_density_varhash: VarHash) {
        // Check if the supplied varhash is a of type "density"
        match &final_density_varhash._type {
            VarHashType::Density => {
                // We can continue
                self.finalized = true;
                FinalDensity::default().new(vec![final_density_varhash], self);
            }
            _ => {
                /* No good */
                panic!()
            }
        }
    }
    // Read back the GLSL data from this interpreter
    pub fn read_glsl(&self) -> Option<String> {
        // Check if we finalized
        if !self.finalized {
            return None;
        }
        //lines.reverse();
        return Some(self.lines.join("\n"));
    }
    // Read back the bound intersection AABB for this interpreter
    pub fn read_aabb(&self) -> Option<AABB> {
        None
    }
}
