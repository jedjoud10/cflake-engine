use std::fmt;

// An error for everything related to the ECS system
#[derive(Debug)]
pub struct ECSError {
    details: String,
}

impl ECSError {
    pub fn new(msg: &str) -> Self {
        Self {
            details: msg.to_string(),
        }
    }
}

impl fmt::Display for ECSError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{}", self.details)
    }
}

impl std::error::Error for ECSError {
    fn description(&self) -> &str {
        &self.details
    }
}
