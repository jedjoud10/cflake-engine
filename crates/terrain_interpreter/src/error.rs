use std::fmt;
use crate::NodeInterpreter;

// A simple interpreter error
#[derive(Debug)]
pub struct InterpreterError {
    details: String,
    node_error: String, 
}

impl InterpreterError {
    pub fn new<T: NodeInterpreter + std::fmt::Debug>(msg: &str, node: T) -> Self {
        Self { details: msg, node_error: format!("{:?}", node) }
    }
}

impl fmt::Display for InterpreterError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "Node: {}", self.node_error)?;
        write!(f, "Error: {}", self.details)?;
    }
}

impl std::error::Error for InterpreterError {
    fn description(&self) -> &str {
        &self.details
    }
}