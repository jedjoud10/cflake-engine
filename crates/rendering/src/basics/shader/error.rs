use std::fmt;

// Shader compilation error
#[derive(Debug, Clone)]
pub struct IncludeExpansionError {
    details: String,
}

impl IncludeExpansionError {
    pub fn new(details: String) -> Self {
        Self { details }
    }
}

impl fmt::Display for IncludeExpansionError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{}", self.details)
    }
}

impl std::error::Error for IncludeExpansionError {
    fn description(&self) -> &str {
        &self.details
    }
}
