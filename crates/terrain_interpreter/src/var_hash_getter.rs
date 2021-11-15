use crate::{
    error::InterpreterError,
    var_hash::{VarHash, VarHashType},
};

// Easier and cleaner way to get var hashes
#[derive(Clone, Debug)]
pub struct VarHashGetter {
    pub inputs: Vec<VarHash>,
    pub inputs_indices: Vec<usize>,
    pub self_varhash: Option<VarHash>,
}

impl VarHashGetter {
    // Get a var
    pub fn get(&self, index: usize, _type: VarHashType) -> Result<VarHash, InterpreterError> {
        let v = self.inputs.get(index).cloned().ok_or(InterpreterError::missing_input(index))?;
        Self::check_type(v, _type).ok_or(InterpreterError::input_err(&v, index, _type))?;
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
