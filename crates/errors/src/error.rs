use std::fmt;

// An error for everything related to the ECS system
#[derive(Debug)]
pub struct ECSError {
    details: String,
}

impl ECSError {
    pub fn new(msg: String) -> Self {
        Self { details: msg }
    }
    pub fn new_str(msg: &str) -> Self {
        Self { details: msg.to_string() }
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

// Resource error
#[derive(Debug)]
pub struct ResourceError {
    details: String,
}

impl ResourceError {
    pub fn new(msg: String) -> Self {
        Self { details: msg }
    }
    pub fn new_str(msg: &str) -> Self {
        Self { details: msg.to_string() }
    }
}

impl fmt::Display for ResourceError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{}", self.details)
    }
}

impl std::error::Error for ResourceError {
    fn description(&self) -> &str {
        &self.details
    }
}

// Rendering error
#[derive(Debug, Clone)]
pub struct RenderingError {
    details: String,
}

impl RenderingError {
    pub fn new(msg: String) -> Self {
        Self { details: msg }
    }
    pub fn new_str(msg: &str) -> Self {
        Self { details: msg.to_string() }
    }
}

impl fmt::Display for RenderingError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{}", self.details)
    }
}

impl std::error::Error for RenderingError {
    fn description(&self) -> &str {
        &self.details
    }
}
