use crate::{Influence, var_hash::VarHash};

// A singular node that consists of a position and an exit density
pub trait NodeInterpreter {
    // Get the string that defines this node
    fn get_node_string(&self, inputs: Vec<VarHash>) -> String;
    // Calculate the influence of this node
    fn calculate_influence(&self) -> Influence;
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
    pub node_interpreter: Box<dyn NodeInterpreter>
}

impl Node {
    // Create a node from inputs and a node interpreter
    pub fn new<T: NodeInterpreter + 'static>(strength: f32, inputs: Vec<VarHash>, node_interpreter: T) -> Self {
        let boxed = Box::new(node_interpreter);
        Self {
            strength,
            inputs,
            output: Vec::new(),
            node_interpreter: boxed,
        }
    }
    // Create the base node for the interpreter
    pub fn new_base<T: NodeInterpreter + Default + 'static>() -> Self {
        let boxed = Box::new(T::default());
        Self {
            strength: 1.0,
            inputs: Vec::new(),
            output: Vec::new(),
            node_interpreter: boxed,
        }
    }
}