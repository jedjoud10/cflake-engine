use crate::{basics::shader::Shader, pipeline::Pipeline};
use super::{Uniforms, ShaderUniformsSettings, UniformError};

// Called whenever we will execute the shader, and we must set the uniforms
#[derive(Default)]
pub struct SetUniformsCallback {
    inner: Option<Box<dyn Fn(&Uniforms) -> Result<(), UniformError> + Send + Sync>>
}

impl SetUniformsCallback {
    // Create a new callback using a closure
    pub fn new<F: Fn(&Uniforms) -> Result<(), UniformError>  + Send + Sync + 'static>(closure: F) -> Self {
        Self { inner: Some(Box::new(closure)) }
    }
    // Execute the callback
    pub(crate) fn execute(&self, uniforms: &Uniforms) {
        if let Some(callback) = &self.inner {
            callback(uniforms);
        } else { panic!("Tried to set uniforms, but no callback was found!") }
    }
}