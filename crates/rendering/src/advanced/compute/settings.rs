use crate::basics::uniforms::SetUniformsCallback;

// Some compute shader settings that we can use whenever we want to execute a compute shader
pub struct ComputeShaderExecutionSettings {
    // We must know the axii groups
    pub axii: (u16, u16, u16),
    // Callback to set some uniforms
    pub callback: SetUniformsCallback,
}
