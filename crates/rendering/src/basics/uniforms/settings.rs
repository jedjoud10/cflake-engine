use crate::{advanced::compute::ComputeShader, basics::shader::Shader, object::ObjectID, pipeline::Pipeline};

// Some type of shader identifier we can use to execute a shader
#[derive(Clone, Copy)]
pub enum ShaderIDType {
    // The ID of a specific shader, if available
    ObjectID(ObjectID<Shader>),
    // The ID of a specific compute shader, if available
    ComputeObjectID(ObjectID<ComputeShader>),
    // The ID of a specific OpenGL program, if available
    OpenGLID(u32),
}

impl ShaderIDType {
    // Get the program OID of the shader
    pub(crate) fn get_program(&self, pipeline: &Pipeline) -> u32 {
        match self {
            ShaderIDType::ObjectID(shader_id) => pipeline.shaders.get(*shader_id).unwrap().program,
            ShaderIDType::ComputeObjectID(compute_shader_id) => pipeline.compute_shaders.get(*compute_shader_id).unwrap().program,
            ShaderIDType::OpenGLID(program) => *program,
        }
    }
}


// Stores the current shader and the shader ID possibly of the shader linked to the uniforms
#[derive(Clone, Copy)]
pub struct ShaderUniformsSettings {
    pub(crate) _type: ShaderIDType,
}

impl ShaderUniformsSettings {
    // Create some new uniform settings using a shader ID type
    pub fn new(_type: ShaderIDType) -> Self {
        Self { _type }
    }
}
