use crate::{Influence, NodeInterpreter, error::InterpreterError, var_hash::{VarHash, VarHashType}, var_hash_getter::VarHashGetter};

// Final density
#[derive(Default, Debug)]
pub struct FinalDensity();

impl NodeInterpreter for FinalDensity {
    // Custom name
    fn custom_name(&self, name: String) -> String {
        format!("{}", "final_density".to_string())
    }
    fn get_node_string(&self, getter: &VarHashGetter) -> Result<String, InterpreterError> {
        // Create the GLSL string for this node, so we can make a variable out of it
        let input = getter.get(0, VarHashType::Density)?;
        Ok(input.get_name())
    }
    fn get_output_type(&self, getter: &VarHashGetter) -> VarHashType {
        VarHashType::Density
    }
}
