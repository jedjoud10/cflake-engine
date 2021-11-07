use crate::{
    error::InterpreterError,
    var_hash::{VarHash, VarHashType},
    Influence, NodeInterpreter,
};
#[derive(Debug)]
pub enum DensityOperationType {
    Union,
    Intersection,
}

impl NodeInterpreter for DensityOperationType {
    fn get_node_string(&self, inputs: &Vec<VarHash>) -> Result<String, InterpreterError> {
        // Check if we have the right amount of inputs
        if inputs.len() != 2 {
            return Err(InterpreterError::missing_input(1, self));
        }
        // Check if we are using density inputs in the first place
        for (i, x) in inputs.iter().enumerate() {
            match x._type {
                crate::var_hash::VarHashType::Density => {} /* This is what we want */
                _ => {
                    return Err(InterpreterError::input_err(x, i, self, VarHashType::Density));
                }
            }
        }
        // Get the GLSL name of the operation and combine with the two inputs
        Ok(match self {
            DensityOperationType::Union => format!("min({}, {})", inputs[0].get_name(), inputs[1].get_name()),
            DensityOperationType::Intersection => format!("max({}, -{})", inputs[0].get_name(), inputs[1].get_name()),
        })
    }
    fn get_output_type(&self) -> crate::var_hash::VarHashType {
        crate::var_hash::VarHashType::Density
    }
}
