use crate::{Influence, Interpreter, var_hash::{VarHash, VarHashType}};

// A singular node that consists of a position and an exit density
pub trait NodeInterpreter {
    // Creata a new node
    fn new(self, inputs: Vec<VarHash>, interpreter: &mut Interpreter) -> VarHash where Self: Sized {
        // Add
        interpreter.add(self, inputs)
    }
    // Get the string that defines this node
    fn get_node_string(&self, inputs: Vec<VarHash>) -> String;
    // Calculate the influence of this node
    fn calculate_influence(&self) -> Influence;
    // Get the output varhash type
    fn get_output_type(&self) -> VarHashType {
        VarHashType::Density
    }
    // Custom name
    fn custom_name(&self, name: String) -> String {
        // Default is passthrough
        name
    }
}