use crate::{error::InterpreterError, var_hash::VarHashType, var_hash_getter::VarHashGetter, NodeInterpreter};
pub enum DensityOperation {
    Union,
    Intersection,
    // We can only do this if we know the influence of the inputs
    Addition,
    Subtraction,
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
            DensityOperation::Subtraction => format!("{} - {}", i0, i1),
        })
    }
    fn get_output_type(&self, _getter: &VarHashGetter) -> crate::var_hash::VarHashType {
        crate::var_hash::VarHashType::Density
    }
    fn calculate_range(&self, getter: &VarHashGetter, input_ranges: Vec<(f32, f32)>) -> (f32, f32) {
        // Depends on the density operation
        match self {
            DensityOperation::Union => todo!(),
            DensityOperation::Intersection => todo!(),
            DensityOperation::Addition => {
                // Take the maximum and minimum
                let (x1, y1) = input_ranges[0];
                let (x2, y2) = input_ranges[1];
                let new_range = (f32::min(x1, x2), f32::max(y1, y2));
                new_range
            },
            DensityOperation::Subtraction => todo!(),
        }
    }
}
