use crate::SubShaderType;

// Cooler objects
pub struct ModelGPUObject(pub u32);
pub struct SubShaderGPUObject(pub SubShaderType, pub u32);
pub struct ShaderGPUObject(pub u32);
pub struct TextureGPUObject(pub u32);



// Some identifiers that we will use to communicate from the Render Thread -> Main Thread
pub enum GPUObject {
    None,           // This value was not initalized yet
    Model(ModelGPUObject),   // The VAO ID
    SubShader(SubShaderGPUObject), // The subshader program ID
    Shader(ShaderGPUObject),  // The shader program ID
    Texture(TextureGPUObject), // The texture ID
}

impl Default for GPUObject {
    fn default() -> Self {
        Self::None
    }
}
