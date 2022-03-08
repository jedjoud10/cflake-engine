use super::Uniforms;
use std::sync::Arc;
// Called whenever we will execute a shader, and we must set it's uniforms

pub type SetUniformsFunction = Box<dyn Fn(&mut Uniforms) + Sync + Send>;
#[derive(Default)]
pub struct StoredUniforms {
    inner: Option<SetUniformsFunction>,
}

impl StoredUniforms {
    // Create a new callback using a closure
    pub fn new<F: Fn(&mut Uniforms) + Sync + Send + 'static>(closure: F) -> Self {
        Self { inner: Some(Box::new(closure)) }
    }

    // Execute the stored uniforms
    pub fn execute(&self, uniforms: &mut Uniforms) {
        if let Some(inner) = &self.inner {
            inner(uniforms)
        }
    }
}
