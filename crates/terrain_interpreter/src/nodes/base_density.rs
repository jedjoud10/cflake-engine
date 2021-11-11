use crate::{error::InterpreterError, var_hash::VarHashType, var_hash_getter::VarHashGetter, NodeInterpreter};

// Base density, the starting point of the actual mesh.
// Must be inputted with a "shape" density first
#[derive(Default)]
pub struct BaseDensity();

impl NodeInterpreter for BaseDensity {
    fn get_node_string(&self, getter: &VarHashGetter) -> Result<String, InterpreterError> {
        // This must be a shape node
        let i0 = getter.get(0, VarHashType::Density)?;
        // Just a pass through lol
        Ok(i0.get_name())
    }
    fn get_output_type(&self, _getter: &VarHashGetter) -> VarHashType {
        VarHashType::Density
    }
}
