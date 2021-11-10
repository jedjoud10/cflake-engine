use crate::{error::InterpreterError, var_hash::VarHashType, var_hash_getter::VarHashGetter, NodeInterpreter};
pub enum VectorOperations {
    Length,
    Dot,
    Multiplication,
    Addition,
}

impl NodeInterpreter for VectorOperations {
    fn get_node_string(&self, getter: &VarHashGetter) -> Result<String, InterpreterError> {
        // Check if we have the right amount of inputs
        let i0 = getter.get(0, VarHashType::Vec3)?.get_name();
        let i1 = getter.get(1, VarHashType::Vec3);
        // Get the GLSL name of the operation and combine with the two inputs
        Ok(match self {
            VectorOperations::Length => format!("length({})", i0),
            VectorOperations::Dot => format!("dot({}, {})", i0, i1?.get_name()),
            VectorOperations::Multiplication => format!("{} * {}", i0, i1?.get_name()),
            VectorOperations::Addition => format!("{} + {}", i0, i1?.get_name()),
        })
    }
    fn get_output_type(&self, _getter: &VarHashGetter) -> VarHashType {
        // Depends on the vector operation
        match self {
            VectorOperations::Length => VarHashType::Density,
            VectorOperations::Dot => VarHashType::Density,
            VectorOperations::Multiplication => VarHashType::Vec3,
            VectorOperations::Addition => VarHashType::Vec3,
        }
    }
}
