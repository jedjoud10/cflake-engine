use crate::{Influence, NodeInterpreter, var_hash::VarHash};
pub enum DensityOperationType {
    Union,
    Intersection,
}

impl NodeInterpreter for DensityOperationType {
    fn get_node_string(&self, inputs: &Vec<VarHash>) -> Result<String, InterpreterError> {
        // Check if we are using density inputs in the first place
        if inputs.iter().any(|x| match x._type {
            crate::var_hash::VarHashType::Density => false /* This is what we want */,
            _ => true
        }) { panic!() }
        // Get the GLSL name of the operation and combine with the two inputs
        match &self {
            DensityOperationType::Union => format!("min({}, {})", inputs[0].get_name(), inputs[1].get_name()),
            DensityOperationType::Intersection => format!("max({}, -{})", inputs[0].get_name(), inputs[1].get_name()),
        }
    }

    fn calculate_influence(&self, influence_inputs: &Vec<Influence>) -> Influence {
        // If one of the inputs has an Influence::Default, then we must sum the influence
        // Otherwise, we must take the max/min (corresponding to our current DensityOperationType)
        if influence_inputs.iter().any(|x| match x {
            Influence::None => todo!(),
            Influence::Default => todo!(),
            Influence::Modified(_, _) => todo!(),
        }) {

        }
        todo!()
    }

    fn get_output_type(&self) -> crate::var_hash::VarHashType {
        crate::var_hash::VarHashType::Density
    }
}