use math::constructive_solid_geometry::CSGTree;

use crate::{
    error::InterpreterError,
    var_hash::{VarHash, VarHashType},
    var_hash_getter::VarHashGetter,
    Interpreter,
};

pub struct Node {
    // Le bruh
    pub getter: VarHashGetter,
    pub node_interpreter: Box<dyn NodeInterpreter>,
}

// A singular node that consists of a position and an exit density
pub trait NodeInterpreter {
    // Custom name
    fn custom_name(&self, name: String) -> String {
        // Default is passthrough
        name
    }
    // Get the string that defines this node
    fn get_node_string(&self, inputs: &VarHashGetter) -> Result<String, InterpreterError>;
    // Get the output varhash type
    fn get_output_type(&self, getter: &VarHashGetter) -> VarHashType;
    // Creata a new node
    fn new(self, inputs: &[VarHash], interpreter: &mut Interpreter) -> Result<VarHash, InterpreterError>
    where
        Self: Sized + 'static,
    {
        // Create the getter
        let getter = VarHashGetter {
            inputs: inputs.to_vec(),
            inputs_indices: inputs.iter().map(|x| x.index).collect(),
            self_varhash: None,
        };
        // Add
        interpreter.add(self, getter)
    }
    // Get the influence of a specific node using it's inputs
    fn update_csgtree(&self, getter: &VarHashGetter, csgtree: &mut CSGTree, self_range: (f32, f32)) {}
    // Calculate the possible range for this node
    fn calculate_range(&self, getter: &VarHashGetter, input_ranges: Vec<(f32, f32)>) -> (f32, f32) { (0.0, 0.0) }
}
