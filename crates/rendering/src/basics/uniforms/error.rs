use std::fmt;

// uniform error error
#[derive(Debug)]
pub struct UniformError {
    uniform: String,
    cause: String,
}

impl UniformError {
    pub fn new(uniform: &str, cause: &str) -> Self {
        Self { uniform: uniform.to_string(), cause: cause.to_string() }
    }
    pub fn invalid_location(uniform: &str) -> Self {
        Self {
            uniform: uniform.to_string(), cause: "invalid location".to_string()
        }
    }
}

impl fmt::Display for UniformError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "could not set uniform '{}' because of an {}!", self.uniform, self.cause)
    }
}

impl std::error::Error for UniformError {
    fn description(&self) -> &str {
        &self.uniform
    }
}
