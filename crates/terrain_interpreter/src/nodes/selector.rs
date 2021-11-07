use crate::{
    error::InterpreterError,
    var_hash::{VarHashType},
    var_hash_getter::VarHashGetter, NodeInterpreter,
};

// A selector node
#[derive(Debug)]
pub struct Selector();

impl NodeInterpreter for Selector {
    fn get_node_string(&self, getter: &VarHashGetter) -> Result<String, InterpreterError> {
        // Check if the two inputs are of type "Float"
        let i0 = getter.get(0, VarHashType::Bool)?.get_name();
        let i1 = getter.get(1, VarHashType::Density)?.get_name();
        let i2 = getter.get(2, VarHashType::Density)?.get_name();
        Ok(format!("{} ? {} : {}", i0, i1, i2))
    }
    fn get_output_type(&self, _getter: &VarHashGetter) -> VarHashType {
        VarHashType::Density
    }
}
