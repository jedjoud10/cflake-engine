use smallvec::SmallVec;

use crate::basics::uniforms::SetUniformsCallback;

// Some compute shader settings that we can use whenever we want to execute a compute shader
pub struct ComputeShaderExecutionSettings {
    // We must know the axii groups
    pub axii: veclib::Vector3<u16>,
    // Callback to set some uniforms
    pub callbacks: SmallVec<[SetUniformsCallback; 1]>,
}

impl ComputeShaderExecutionSettings {
    // Create some new settings using the axii counts
    pub fn new(axii: veclib::Vector3<u16>) -> Self {
        Self {
            axii,
            callbacks: SmallVec::default(),
        }
    }
    // Add a callback
    pub fn with_callback(mut self, callback: SetUniformsCallback) -> Self {
        self.callbacks.push(callback);
        self
    }
}