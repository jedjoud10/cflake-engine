use crate::{advanced::compute::ComputeShader, basics::shader::Shader, object::ObjectID, pipeline::Pipeline};

// Some type of shader identifier we can use to execute a shader
#[derive(Clone, Copy)]
pub enum ShaderIdentifier {
    // The ID of a specific shader, if available
    ObjectID(ObjectID<Shader>),
    // The ID of a specific compute shader, if available
    ComputeObjectID(ObjectID<ComputeShader>),
    // The ID of a specific OpenGL program, if available
    OpenGLID(u32),
}

// Stores the current shader and the shader ID possibly of the shader linked to the uniforms
#[derive(Clone, Copy)]
pub struct ShaderUniformsSettings {
    pub(crate) identifier: ShaderIdentifier,
}

impl ShaderUniformsSettings {
    // Create some new uniform settings using a shader identifier
    pub fn new(identifier: ShaderIdentifier) -> Self {
        Self { identifier }
    }
    // Get the program OID of the shader
    pub(crate) fn get_program_id(&self, pipeline: &Pipeline) -> u32 {
        match self.identifier {
            ShaderIdentifier::ObjectID(x) => pipeline.shaders.get(x).unwrap().program,
            ShaderIdentifier::ComputeObjectID(x) => pipeline.compute_shaders.get(x).unwrap().program,
            ShaderIdentifier::OpenGLID(x) => x,
        }
    }
}
