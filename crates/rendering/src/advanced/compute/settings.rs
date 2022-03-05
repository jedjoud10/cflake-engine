use crate::basics::uniforms::StoredUniforms;

// Some compute shader settings that we can use whenever we want to execute a compute shader
pub struct ComputeShaderExecutionSettings {
    // We must know the axii groups
    pub axii: veclib::Vector3<u16>,
    // Callback to set some uniforms
    pub uniforms: StoredUniforms,
}

impl ComputeShaderExecutionSettings {
    // Create some new settings using the axii counts
    pub fn new(axii: veclib::Vector3<u16>, uniforms: StoredUniforms) -> Self {
        Self {
            axii,
            uniforms,
        }
    }
}
