use crate::{Influence, NodeInterpreter, error::InterpreterError, var_hash::{VarHash, VarHashType}};

// The base position interpreter
#[derive(Default)]
pub struct BasePosition {
}

impl NodeInterpreter for BasePosition {
    fn get_node_string(&self, inputs: &Vec<VarHash>) -> Result<String, InterpreterError> {
        // Create the GLSL string for this node, so we can make a variable out of it
        Ok("pos".to_string())
    }

    fn calculate_influence(&self, inputs: &Vec<Influence>) -> Influence {
        // Default influence
        Influence::Default
    }

    fn get_output_type(&self) -> VarHashType {
        VarHashType::Vec3
    }
}