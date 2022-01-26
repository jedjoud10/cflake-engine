use super::ComputeShader;
use crate::{basics::uniforms::ShaderUniformsGroup, object::ObjectID};

// Some compute shader settings that we can use whenever we want to execute a compute shader
pub struct ComputeShaderExecutionSettings {
    // We must have the ID of the compute shader
    pub(crate) id: ObjectID<ComputeShader>,
    // We must know the axii groups
    pub(crate) axii: (u16, u16, u16),
    // Store some shader uniforms, if we want to
    pub(crate) uniforms: Option<ShaderUniformsGroup>,
}

impl ComputeShaderExecutionSettings {
    // Create some new compute shader execution settings
    pub fn new(id: ObjectID<ComputeShader>, axii: (u16, u16, u16)) -> Self {
        Self { id, axii, uniforms: None }
    }
    // Set the uniforms
    pub fn set_uniforms(mut self, uniforms: ShaderUniformsGroup) -> Self {
        self.uniforms = Some(uniforms);
        self
    }
}
