// Error that gets thrown whenever we mess up when validating the uniforms
pub enum UniformsError {
    // The user forgot to set the value of a uniform
    IncompleteUniform(String),

    // The user forgot to bind a buffer/block
    IncompleteBinding(String),

    // The user tried to set a uniform, but it does not exist
    InvalidUniformName(String),

    // The user tried to set a binding point, but it does not exist
    InvalidBindingName(String),
}

impl std::fmt::Debug for UniformsError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            UniformsError::IncompleteUniform(name) => write!(f, "The uniform '{name}' was not set"),
            UniformsError::IncompleteBinding(name) => {
                write!(f, "The binding point '{name}' was not set")
            }
            UniformsError::InvalidUniformName(name) => write!(
                f,
                "Tried to set uniform '{name}', but it does not exist in the program"
            ),
            UniformsError::InvalidBindingName(name) => write!(
                f,
                "Tried to set binding point '{name}', but it does not exist in the program"
            ),
        }
    }
}

impl std::fmt::Display for UniformsError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        std::fmt::Debug::fmt(&self, f)
    }
}

impl std::error::Error for UniformsError {}
