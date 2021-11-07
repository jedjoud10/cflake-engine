use crate::{
    error::InterpreterError,
    var_hash::{VarHashType},
    var_hash_getter::VarHashGetter, NodeInterpreter,
};

// Final density
#[derive(Default, Debug)]
pub struct FinalDensity();

impl NodeInterpreter for FinalDensity {
    // Custom name
    fn custom_name(&self, _name: String) -> String {
        "final_density".to_string()
    }
    fn get_node_string(&self, getter: &VarHashGetter) -> Result<String, InterpreterError> {
        // Create the GLSL string for this node, so we can make a variable out of it
        let input = getter.get(0, VarHashType::Density)?;
        Ok(input.get_name())
    }
    fn get_output_type(&self, _getter: &VarHashGetter) -> VarHashType {
        VarHashType::Density
    }
}
