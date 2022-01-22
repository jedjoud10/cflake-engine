use crate::{
    error::InterpreterError,
    var_hash::{PassedData, VarHashType},
    var_hash_getter::VarHashGetter,
    NodeInterpreter,
};
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
    fn calculate_range(&self, _getter: &VarHashGetter, input_ranges: Vec<(f32, f32)>) -> (f32, f32) {
        // Depends on the density operation
        match self {
            DensityOperation::Union => {
                // You don't have range in this
                (0.0, 0.0)
            }
            DensityOperation::Intersection => todo!(),
            DensityOperation::Addition => {
                // Take the maximum and minimum
                let (x1, y1) = input_ranges[0];
                let (x2, y2) = input_ranges[1];

                (f32::min(x1, x2), f32::max(y1, y2))
            }
            DensityOperation::Subtraction => todo!(),
        }
    }
    fn update_csgtree(&self, passed_data: &mut PassedData, getter: &VarHashGetter, csgtree: &mut math::constructive_solid_geometry::CSGTree, self_range: (f32, f32)) {
        // Depends on the density operation
        match self {
            DensityOperation::Union => {}
            DensityOperation::Intersection => todo!(),
            DensityOperation::Addition => {
                // Update the CSG tree
                let i0 = getter.get(0, VarHashType::Density).unwrap();
                let i1 = getter.get(1, VarHashType::Density).unwrap();
                let x: PassedData = PassedData::combine(i0.passed_data, i1.passed_data);
                *passed_data = x;
                let custom_shape = csgtree.get_custom_mut(x.custom_shape_identifier.unwrap()).unwrap();
                custom_shape.expand(math::csg::ExpandMethod::Factor(self_range.1));
            }
            DensityOperation::Subtraction => todo!(),
        }
    }
}
