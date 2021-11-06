use crate::{Influence, NodeInterpreter, var_hash::VarHash};

// The base position interpreter
#[derive(Default)]
pub struct BasePosition {
}

impl NodeInterpreter for BasePosition {
    fn get_node_string(&self, inputs: Vec<VarHash>) -> String {
        // Create the HLSL string for this node, so we can make a variable out of it
        "pos".to_string()
    }

    fn calculate_influence(&self) -> Influence {
        // Default influence
        Influence::Default
    }

    fn get_output_type(&self) -> crate::var_hash::VarHashType {
        crate::var_hash::VarHashType::Vec3
    }
}