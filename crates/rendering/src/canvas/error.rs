use crate::prelude::UniformsError;

// Error that gets thrown whenever we fail doing something when we try to rasterize an object
pub enum RasterError {
    // We goofed when setting the uniforms
    Uniforms(UniformsError)
}

impl std::fmt::Debug for RasterError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            RasterError::Uniforms(err) => <UniformsError as std::fmt::Debug>::fmt(err, f),
        }
    }
}

impl std::fmt::Display for RasterError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        std::fmt::Debug::fmt(&self, f)
    }
}
impl std::error::Error for RasterError {}
