use crate::basics::shader::Shader;
use super::Uniforms;

// Called whenever we will execute the shader, and we must set the uniforms
pub struct SetUniformsCallback {
    inner: Box<dyn Fn(&Uniforms) + Send + Sync>
}

impl SetUniformsCallback {
    // Create a new callback using a closure
    pub fn new<F: Fn(&Uniforms) + Send + Sync + 'static>(closure: F) -> Self {
        Self { inner: Box::new(closure) }
    }
    // Execute the callback
    pub(crate) fn execute(&self, program: u32) {
    }
}