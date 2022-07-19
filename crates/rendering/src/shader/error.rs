// Error that gets thrown whenever we mess up when validating the uniforms
pub enum UniformsError {
    IncompleteUniform(String),
    IncompleteBufferBinding(String),
    InvalidTexture(String),
    InvalidBuffer(String),
}

impl std::fmt::Debug for UniformsError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            UniformsError::IncompleteUniform(name) => write!(f, "The uniform '{name}' was not set"),
            UniformsError::IncompleteBufferBinding(name) => {
                write!(f, "The binding buffer point '{name}' was not set")
            }
            UniformsError::InvalidTexture(name) => {
                write!(f, "The texture at location '{name}' was destroyed")
            }
            UniformsError::InvalidBuffer(name) => {
                write!(f, "The buffer at binding '{name}' was destroyed")
            }
        }
    }
}

impl std::fmt::Display for UniformsError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        std::fmt::Debug::fmt(&self, f)
    }
}

impl std::error::Error for UniformsError {}
