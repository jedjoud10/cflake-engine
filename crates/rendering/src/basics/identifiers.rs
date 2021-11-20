use crate::SubShaderType;

// Some identifiers that we will use to communicate from the Render Thread -> Main Thread
#[derive(Clone)]
pub enum GPUObject {
    None,           // This value was not initalized yet
    Model(u32),   // The VAO ID
    SubShader(SubShaderType, u32), // The subshader program ID
    Shader(u32),  // The shader program ID
    Texture(u32), // The texture ID
}

impl Default for GPUObject {
    fn default() -> Self {
        Self::None
    }
}
