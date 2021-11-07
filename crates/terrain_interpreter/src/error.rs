use crate::{
    var_hash::{VarHash, VarHashType},
    NodeInterpreter,
};
use std::fmt;

// A simple interpreter error
#[derive(Debug)]
pub struct InterpreterError {
    details: String,
}

impl InterpreterError {
    pub fn new(msg: &str) -> Self {
        Self {
            details: msg.to_string(),
        }
    }
    // Create an input error
    pub fn input_err(input: &VarHash, index: usize, expected: VarHashType) -> Self {
        Self {
            details: format!("Input '{:?}' at index '{}' is invalid! Expected VarHashType '{:?}'!", input, index, expected),
        }
    }
    // Create a missing input error
    pub fn missing_input(index: usize) -> Self {
        Self {
            details: format!("Input at index '{}' is missing!", index),
        }
    }
}

impl fmt::Display for InterpreterError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "Error: {}", self.details)
    }
}

impl std::error::Error for InterpreterError {
    fn description(&self) -> &str {
        &self.details
    }
}
