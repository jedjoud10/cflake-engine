// Error that gets thrown whenever we mess up when validating the uniforms
pub enum UniformsError {
    IncompleteUniform(String),
    IncompleteBufferBinding(String),
    DeletedTextureUnit(String),
    DeletedBufferBinding(String),
}

impl std::fmt::Debug for UniformsError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            UniformsError::IncompleteUniform(name) => {
                write!(f, "The uniform '{name}' was not set")
            }
            UniformsError::IncompleteBufferBinding(name) => {
                write!(f, "The binding buffer point '{name}' was not set")
            }
            UniformsError::DeletedTextureUnit(name) => {
                write!(f, "The texture at the texture unit '{name}' was deleted")
            }
            UniformsError::DeletedBufferBinding(name) => {
                write!(
                    f,
                    "The buffer at the buffer binding point '{name}' was deleted"
                )
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
