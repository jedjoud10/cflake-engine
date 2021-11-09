use crate::{NodeInterpreter, error::InterpreterError, var_hash::VarHashType, var_hash_getter::VarHashGetter};

// The base position interpreter
#[derive(Default)]
pub struct BasePosition();

impl NodeInterpreter for BasePosition {
    fn get_node_string(&self, _getter: &VarHashGetter) -> Result<String, InterpreterError> {
        // Create the GLSL string for this node, so we can make a variable out of it
        Ok("pos".to_string())
    }
    fn get_output_type(&self, _getter: &VarHashGetter) -> VarHashType {
        VarHashType::Vec3
    }
}
