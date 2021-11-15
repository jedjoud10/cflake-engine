use crate::{error::InterpreterError, var_hash::VarHashType, var_hash_getter::VarHashGetter, NodeInterpreter};

// A fractal node
pub enum Fractal {
    FBM(u8, f32, f32)
}

impl NodeInterpreter for Fractal {
    fn get_node_string(&self, getter: &VarHashGetter) -> Result<String, InterpreterError> {
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
