use std::fmt;

// Created whenever we tried to access some values about a non initialized pipeline object
#[derive(Debug, Clone)]
pub struct OpenGLObjectNotInitialized;

impl fmt::Display for OpenGLObjectNotInitialized {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "OpenGL object not initialized!")
    }
}

impl std::error::Error for OpenGLObjectNotInitialized {}
