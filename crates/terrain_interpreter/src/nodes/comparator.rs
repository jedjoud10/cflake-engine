use crate::{Influence, NodeInterpreter, error::InterpreterError, var_hash::{VarHash, VarHashType}};

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
    fn get_node_string(&self, inputs: &Vec<VarHash>) -> Result<String, InterpreterError> {
        // Check if the two inputs are of type "Float"
        let i0 = inputs.get(0).ok_or(InterpreterError::missing_input(0, self))?;
        let i1 = inputs.get(1).ok_or(InterpreterError::missing_input(1, self))?;
        match i0._type {
            VarHashType::Bool => return Err(InterpreterError::input_err(i0, 0, self, VarHashType::Float)),
            _ => {}
        }
        match i1._type {
            VarHashType::Bool => return Err(InterpreterError::input_err(i1, 1, self, VarHashType::Float)),
            _ => {}
        }
        let i0 = i0.get_name();
        let i1 = i1.get_name();
        Ok(match self {
            Comparator::Equal => format!("{} == {}", i0, i1),
            Comparator::LessThan => format!("{} < {}", i0, i1),
            Comparator::GreaterThan => format!("{} > {}", i0, i1),
            Comparator::LessThanEqual => format!("{} <= {}", i0, i1),
            Comparator::GreaterThanEqual => format!("{} >= {}", i0, i1),
        })
    }

    fn calculate_influence(&self, inputs: &Vec<Influence>) -> Influence {
        // Max possible influence
        Influence::None
    }

    fn get_output_type(&self) -> VarHashType {
        VarHashType::Bool
    }
}