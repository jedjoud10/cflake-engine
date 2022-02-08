use std::fmt;

#[derive(Debug, Clone)]
pub struct RenderingError {
    renderer_id: ObjectID<Renderer>,
    model_id: ObjectID<Renderer>,
    material_id: ObjectID<Renderer>,
}

impl fmt::Display for OpenGLObjectNotInitialized {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "OpenGL object not initialized!")
    }
}

impl std::error::Error for OpenGLObjectNotInitialized {}
