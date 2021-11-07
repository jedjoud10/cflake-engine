use std::fmt;
use crate::{NodeInterpreter, var_hash::{VarHash, VarHashType}};

// A simple interpreter error
#[derive(Debug)]
pub struct InterpreterError {
    details: String,
    node_error: String, 
}

impl InterpreterError {
    pub fn new<T: NodeInterpreter + std::fmt::Debug>(msg: &str, node: T) -> Self {
        Self { details: msg.to_string(), node_error: format!("{:?}", node) }
    }
    // Create an input error
    pub fn input_err<T: NodeInterpreter + std::fmt::Debug>(input: &VarHash, index: usize, node: &T, expected: VarHashType) -> Self {
        Self {
            details: format!("Input '{:?}' at index '{}' is invalid! Expected VarHashType '{:?}'!", input, index, expected),
            node_error: format!("{:?}", node),
        }
    }
    // Create a missing input error
    pub fn missing_input<T: NodeInterpreter + std::fmt::Debug>(index: usize, node: &T) -> Self {
        Self {
            details: format!("Input at index '{}' is missing!", index),
            node_error: format!("{:?}", node),
        }
    }
}

impl fmt::Display for InterpreterError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "Node: {}", self.node_error)?;
        write!(f, "Error: {}", self.details)
    }
}

impl std::error::Error for InterpreterError {
    fn description(&self) -> &str {
        &self.details
    }
}