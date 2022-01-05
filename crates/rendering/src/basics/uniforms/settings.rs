// Stores the current shader and the shader ID possibly of the shader linked to the uniforms
pub struct ShaderUniformsSettings {
    shader_id: ObjectID<Shader>,
}

impl ShaderUniformsSettings {
    // Create some new uniform settings using a shader ID
    pub fn new_id(shader_id: ObjectID<Shader>) -> Self {
        Self {
            shader_id
        }
    }
}

