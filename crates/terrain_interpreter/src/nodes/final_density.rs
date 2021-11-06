use crate::{Influence, NodeInterpreter, var_hash::{VarHash, VarHashType}};

// Final density
#[derive(Default)]
pub struct FinalDensity {
}

impl NodeInterpreter for FinalDensity {
    fn get_node_string(&self, inputs: Vec<VarHash>) -> String {
        // Create the HLSL string for this node, so we can make a variable out of it
        inputs[0].get_name()
    }

    fn calculate_influence(&self) -> Influence {
        // Default influence
        Influence::None
    }
    // Custom name
    fn custom_name(&self, name: String) -> String {
        format!("{}_{}", "final_density".to_string(), name)
    }
}