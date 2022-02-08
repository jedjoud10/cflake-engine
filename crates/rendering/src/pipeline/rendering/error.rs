use std::fmt;

#[derive(Debug, Clone)]
pub struct RenderingError;

impl fmt::Display for RenderingError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "Drawing failed!")
    }
}

impl std::error::Error for RenderingError {}
