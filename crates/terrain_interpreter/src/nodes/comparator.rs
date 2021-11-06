use crate::{Influence, NodeInterpreter, var_hash::{VarHash, VarHashType}};

// A comparator node (if)
pub enum Comparator {
    Equal,
    LessThan,
    GreaterThan,
    LessThanEqual,
    GreaterThanEqual,
}

impl NodeInterpreter for Comparator {
    fn get_node_string(&self, inputs: &Vec<VarHash>) -> Result<String, InterpreterError> {
        // We can only compare between variables of the same type, and if they aren't a bool or a density
        if inputs.len() == 2 {

        } else {
            return None
        }
        // We have 2 inputs, we must compare between them
        let i0 = inputs[0].get_name();
        let i1 = inputs[1].get_name();
        match self {
            Comparator::Equal => format!("{} == {}", i0, i1),
            Comparator::LessThan => format!("{} < {}", i0, i1),
            Comparator::GreaterThan => format!("{} > {}", i0, i1),
            Comparator::LessThanEqual => format!("{} <= {}", i0, i1),
            Comparator::GreaterThanEqual => format!("{} >= {}", i0, i1),
        }
    }

    fn calculate_influence(&self, inputs: &Vec<Influence>) -> Influence {
        // Max possible influence
        Influence::None
    }

    fn get_output_type(&self) -> VarHashType {
        VarHashType::Bool
    }
}