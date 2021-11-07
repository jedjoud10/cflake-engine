use crate::{Influence, NodeInterpreter, error::InterpreterError, var_hash::{VarHash, VarHashType}};

// Final density
#[derive(Default, Debug)]
pub struct FinalDensity {
}

impl NodeInterpreter for FinalDensity {
    fn get_node_string(&self, inputs: &Vec<VarHash>) -> Result<String, InterpreterError> {
        // Create the GLSL string for this node, so we can make a variable out of it
        let input = inputs.get(0).ok_or(InterpreterError::missing_input(0, self))?;
        match input._type {
            VarHashType::Density => {},
            _ => { return Err(InterpreterError::input_err(input, 0, self, VarHashType::Density)); }
        }
        Ok(input.get_name())
    }

    fn calculate_influence(&self, inputs: &Vec<Influence>) -> Influence {
        // Default influence
        Influence::None
    }
    // Custom name
    fn custom_name(&self, name: String) -> String {
        format!("{}", "final_density".to_string())
    }
}