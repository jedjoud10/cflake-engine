use std::fmt;

// An error struct for everything related to the entities
#[derive(Debug)]
pub struct EntityError {
    details: String,
}

impl EntityError {
    pub fn new(msg: &str) -> Self {
        Self {
            details: msg.to_string(),
        }
    }
}

impl fmt::Display for EntityError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{}", self.details)
    }
}

impl std::error::Error for EntityError {
    fn description(&self) -> &str {
        &self.details
    }
}

// An error struct for everything related to the systems
#[derive(Debug)]
pub struct SystemError {
    details: String,
}

impl SystemError {
    pub fn new(msg: &str) -> Self {
        Self {
            details: msg.to_string(),
        }
    }
}

impl fmt::Display for SystemError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{}", self.details)
    }
}

impl std::error::Error for SystemError {
    fn description(&self) -> &str {
        &self.details
    }
}

// An error struct for everything related to the components
#[derive(Debug)]
pub struct ComponentError {
    details: String,
}

impl ComponentError {
    pub fn new(msg: &str) -> Self {
        Self {
            details: msg.to_string(),
        }
    }
}

impl fmt::Display for ComponentError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{}", self.details)
    }
}

impl std::error::Error for ComponentError {
    fn description(&self) -> &str {
        &self.details
    }
}
