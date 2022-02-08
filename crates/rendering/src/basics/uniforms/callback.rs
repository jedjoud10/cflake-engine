use super::{ShaderUniformsSettings, UniformError, Uniforms};
use crate::{basics::shader::Shader, pipeline::Pipeline};

// Called whenever we will execute the shader, and we must set the uniforms
#[derive(Default)]
pub struct SetUniformsCallback {
    inner: Option<Box<dyn Fn(&Uniforms) + Send + Sync>>,
}

impl SetUniformsCallback {
    // Create a new callback using a closure
    pub fn new<F: Fn(&Uniforms) + Send + Sync + 'static>(closure: F) -> Self {
        Self { inner: Some(Box::new(closure)) }
    }
    // Execute the callback
    pub(crate) fn execute(&self, uniforms: &Uniforms) {
        if let Some(callback) = &self.inner {
            callback(uniforms);
        } else {
            eprintln!("Tried to set uniforms, but no callback was found!")
        }
    }
}
