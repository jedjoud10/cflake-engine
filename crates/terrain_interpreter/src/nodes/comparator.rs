use crate::{
    error::InterpreterError,
    var_hash::{VarHashType},
    var_hash_getter::VarHashGetter, NodeInterpreter,
};

// A comparator node (if)
#[derive(Debug)]
pub enum Comparator {
    Equal,
    LessThan,
    GreaterThan,
    LessThanEqual,
    GreaterThanEqual,
}

impl NodeInterpreter for Comparator {
    fn get_node_string(&self, getter: &VarHashGetter) -> Result<String, InterpreterError> {
        // Check if the two inputs are of type "Float"
        let i0 = getter.get(0, VarHashType::Density)?.get_name();
        let i1 = getter.get(1, VarHashType::Density)?.get_name();
        Ok(match self {
            Comparator::Equal => format!("{} == {}", i0, i1),
            Comparator::LessThan => format!("{} < {}", i0, i1),
            Comparator::GreaterThan => format!("{} > {}", i0, i1),
            Comparator::LessThanEqual => format!("{} <= {}", i0, i1),
            Comparator::GreaterThanEqual => format!("{} >= {}", i0, i1),
        })
    }
    fn get_output_type(&self, _getter: &VarHashGetter) -> VarHashType {
        VarHashType::Bool
    }
}
