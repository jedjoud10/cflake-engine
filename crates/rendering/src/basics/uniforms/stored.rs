use super::Uniforms;
use std::sync::Arc;
// Called whenever we will execute a shader, and we must set it's uniforms
#[derive(Default)]
pub struct StoredUniforms {
    pub(crate) inner: Option<Box<dyn Fn(&mut Uniforms)>>,
}

impl StoredUniforms {
    // Create a new callback using a closure
    pub fn new<F: Fn(&mut Uniforms) + 'static>(closure: F) -> Self {
        Self {
            inner: Some(Box::new(closure)),
        }
    }
}
