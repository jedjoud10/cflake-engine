use crate::{object::ObjectID, Shader, compute::ComputeShader, Pipeline};

// Stores the current shader and the shader ID possibly of the shader linked to the uniforms
pub struct ShaderUniformsSettings {
    // The ID of a specific shader, if available
    pub(crate) shader_id: Option<ObjectID<Shader>>,
    // The ID of a specific compute shader, if available
    pub(crate) compute_shader_id: Option<ObjectID<ComputeShader>>,
}

impl ShaderUniformsSettings {
    // Create some new uniform settings using a shader ID
    pub fn new(id: ObjectID<Shader>) -> Self {
        Self {
            shader_id: Some(id),
            compute_shader_id: None,
        }
    }
    // Create some new uniform settings using a compute shader ID
    pub fn new_compute(id: ObjectID<ComputeShader>) -> Self {
        Self {
            shader_id: None,
            compute_shader_id: Some(id),
        }
    }
    // Get the program ID of the shader
    pub(crate) fn get_program_id(&self, pipeline: &Pipeline) -> u32 {
        if let Some(x) = self.shader_id {
            return pipeline.get_shader(x).unwrap().program;
        } else if let Some(y) = self.compute_shader_id {
            return pipeline.get_compute_Shader(y).unwrap().program;
        } else { panic!() }
    }
}

