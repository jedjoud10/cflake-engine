use crate::var_hash::VarHash;

// A singular node that consists of a position and an exit density
pub trait NodeInterpreter {
    // Get the input variable hash
    fn get_input_v_hash(&self) -> u64;
    // Get the output variable hash
    // Get the string that defines this node
    fn get_node_string(&self) -> String;
    // Calculate the influence of this node
    fn calculate_influence(&self) -> f32;
}

// A node
pub struct Node {
    // Node variables
    pub strength: f32,
    // Inputs
    pub inputs: Vec<VarHash>,
    // Outputs
    pub output: Vec<VarHash>,
    // Custom interpreter
    pub interpreter: Box<dyn NodeInterpreter>
}