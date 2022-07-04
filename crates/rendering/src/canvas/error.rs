use crate::prelude::UniformsError;

// Error that gets thrown whenever we fail doing something when we try to rasterize an object
pub enum RasterError {
    // We goofed when setting the uniforms
    Uniforms(UniformsError)
}
/*
impl std::fmt::Debug for UniformsError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            UniformsError::MissingUniform(name) => write!(f, "The uniform '{name}' was not set"),
            UniformsError::MissingBinding(name) => write!(f, "The binding point '{name}' was not set"),
        }
    }
}

impl std::fmt::Display for UniformsError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        std::fmt::Debug::fmt(&self, f)
    }
}
impl std::error::Error for UniformsError {}
*/
