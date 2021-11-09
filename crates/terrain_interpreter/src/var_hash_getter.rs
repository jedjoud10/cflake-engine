use crate::{Influence, error::InterpreterError, var_hash::{VarHash, VarHashType}};

// Easier and cleaner way to get var hashes
pub struct VarHashGetter {
    pub inputs: Vec<VarHash>,
    pub influences: Vec<Influence>,
    pub inputs_nodes_indices: Vec<usize>
}

impl VarHashGetter {
    // Get a var
    pub fn get(&self, index: usize, _type: VarHashType) -> Result<VarHash, InterpreterError> {
        let v = self.inputs.get(index).cloned().ok_or(InterpreterError::missing_input(index))?;
        Self::check_type(v, _type).ok_or(InterpreterError::input_err(&v, index, _type))?;
        Ok(v)
    }
    // Get an influence
    pub fn get_influence(&self, index: usize) -> Result<Influence, InterpreterError> {
        let v = self.influences.get(index).cloned().ok_or(InterpreterError::missing_input(index))?;
        Ok(v)
    }
    // Check if a var is of a specific type
    // Return the inputted varhash if the types match, returns none if they don't match
    fn check_type(var: VarHash, _type: VarHashType) -> Option<VarHash> {
        if var._type == _type {
            Some(var)
        } else {
            None
        }
    }
}
