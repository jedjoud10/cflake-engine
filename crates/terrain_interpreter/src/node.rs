use crate::{
    error::InterpreterError,
    var_hash::{VarHash, VarHashType},
    var_hash_getter::VarHashGetter, Interpreter,
};

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
        Self: Sized,
    {
        // Create the getter
        let getter = VarHashGetter { inputs: inputs.to_vec() };
        // Add
        interpreter.add(self, getter)
    }
}
