use std::sync::Arc;

use super::Uniforms;

// Called whenever we will execute the shader, and we must set the uniforms
#[derive(Default, Clone)]
pub struct SetUniformsCallback {
    inner: Option<Arc<Box<dyn Fn(&Uniforms) + Send + Sync>>>,
}

impl SetUniformsCallback {
    // Create a new callback using a closure
    pub fn new<F: Fn(&Uniforms) + Send + Sync + 'static>(closure: F) -> Self {
        Self { inner: Some(Arc::new(Box::new(closure))) }
    }
    // Execute the callback
    pub(crate) fn execute(&self, uniforms: &Uniforms) {
        if let Some(callback) = &self.inner {
            callback(uniforms);
        }
    }
}
