use super::Uniforms;

// Uniforms set
#[derive(Default)]
pub struct UniformsSet(pub(crate) Option<Box<dyn Fn(Uniforms)>>);

impl UniformsSet {
    pub fn new<F: Fn(Uniforms) + 'static>(callback: F) -> Self {
        Self(Some(Box::new(callback)))
    }
    // Execute the uniforms set
    pub fn execute(&self, uniforms: Uniforms) {
        if let Some(boxed) = &self.0 {
            (boxed.as_ref())(uniforms);
        }
    }
}
