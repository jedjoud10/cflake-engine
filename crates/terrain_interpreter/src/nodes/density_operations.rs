use crate::{error::InterpreterError, var_hash::VarHashType, var_hash_getter::VarHashGetter, NodeInterpreter};
#[derive(Debug)]
pub enum DensityOperation {
    Union,
    Intersection,
    // We can only do this if we know the influence of the inputs
    Addition,
}

impl NodeInterpreter for DensityOperation {
    fn get_node_string(&self, getter: &VarHashGetter) -> Result<String, InterpreterError> {
        // Check if we have the right amount of inputs
        let i0 = getter.get(0, VarHashType::Density)?.get_name();
        let i1 = getter.get(1, VarHashType::Density)?.get_name();
        // Get the GLSL name of the operation and combine with the two inputs
        Ok(match self {
            DensityOperation::Union => format!("min({}, {})", i0, i1),
            DensityOperation::Intersection => format!("max({}, -{})", i0, i1),
            DensityOperation::Addition => format!("{} + {}", i0, i1),
        })
    }
    fn get_output_type(&self, _getter: &VarHashGetter) -> crate::var_hash::VarHashType {
        crate::var_hash::VarHashType::Density
    }
}
