use std::collections::HashMap;

use crate::{SubShaderType, TextureShaderAccessType, TextureType, Uniform};

// Cooler objects
#[derive(Clone, Default)]
pub struct ModelGPUObject(pub u32);
#[derive(Clone, Default)]
pub struct SubShaderGPUObject(pub SubShaderType, pub u32);
#[derive(Clone, Default)]
pub struct ShaderGPUObject(pub u32);
#[derive(Clone, Default)]
pub struct ComputeShaderGPUObject(pub u32);
#[derive(Clone, Default)]
pub struct TextureGPUObject(pub u32, pub TextureType);

// Each shader will contain a "shader excecution group" that will contain uniforms that must be sent to the GPU when that shader gets run
pub struct ShaderUniformsGroup {
    // Uniforms
    uniforms: HashMap<String, Uniform>,
}

// Gotta change the place where this shit is in
impl ShaderUniformsGroup {
    // Create a new empty shader excecution group (Used for initial states)
    pub fn new_null() -> Self {
        Self {
            uniforms: HashMap::new(),
        }
    }
    // Set a bool uniform
    pub fn set_bool(&mut self, name: &str, value: bool) {
        self.uniforms.insert(name.to_string(), Uniform::Bool(value));
    }
    // Set a f32 uniform
    pub fn set_f32(&mut self, name: &str, value: f32) {
        self.uniforms.insert(name.to_string(), Uniform::F32(value));
    }
    // Set a 2D image
    pub fn set_i2d(&mut self, name: &str, texture: TextureGPUObject, access_type: TextureShaderAccessType) {
        self.uniforms.insert(name.to_string(), Uniform::Image2D(texture, access_type));
    }
    // Set a i32
    pub fn set_i32(&mut self, name: &str, value: i32) {
        self.uniforms.insert(name.to_string(), Uniform::I32(value));
    }
    // Set a 3D image
    pub fn set_i3d(&mut self, name: &str, texture: TextureGPUObject, access_type: TextureShaderAccessType) {
        self.uniforms.insert(name.to_string(), Uniform::Image3D(texture, access_type));
    }
    // Set a matrix 4x4 f32
    pub fn set_mat44(&mut self, name: &str, matrix: veclib::Matrix4x4<f32>) {
        self.uniforms.insert(name.to_string(), Uniform::Mat44F32(matrix));
    }
    // Set a 1D texture
    pub fn set_t1d(&mut self, name: &str, texture: TextureGPUObject, active_texture_id: u32) {
        self.uniforms.insert(name.to_string(), Uniform::Texture1D(texture, active_texture_id));
    }
    // Set a 2D texture
    pub fn set_t2d(&mut self, name: &str, texture: TextureGPUObject, active_texture_id: u32) {
        self.uniforms.insert(name.to_string(), Uniform::Texture1D(texture, active_texture_id));
    }
    // Set a texture2d array
    pub fn set_t2da(&mut self, name: &str, texture: TextureGPUObject, active_texture_id: u32) {
        self.uniforms.insert(name.to_string(), Uniform::Texture2DArray(texture, active_texture_id));
    }
    // Set a 3D texture
    pub fn set_t3d(&mut self, name: &str, texture: TextureGPUObject, active_texture_id: u32) {
        self.uniforms.insert(name.to_string(), Uniform::Texture3D(texture, active_texture_id));
    }
    // Set a vec2 f32 uniform
    pub fn set_vec2f32(&mut self, name: &str, vec: veclib::Vector2<f32>) {
        self.uniforms.insert(name.to_string(), Uniform::Vec2F32(vec));
    }
    // Set a vec2 i32 uniform
    pub fn set_vec2i32(&mut self, name: &str, vec: veclib::Vector2<i32>) {
        self.uniforms.insert(name.to_string(), Uniform::Vec2I32(vec));
    }
    // Set a vec3 f32 uniform
    pub fn set_vec3f32(&mut self, name: &str, vec: veclib::Vector3<f32>) {
        self.uniforms.insert(name.to_string(), Uniform::Vec3F32(vec));
    }
    // Set a vec3 i32 uniform
    pub fn set_vec3i32(&mut self, name: &str, vec: veclib::Vector3<i32>) {
        self.uniforms.insert(name.to_string(), Uniform::Vec3I32(vec));
    }
    // Set a vec4 f32 uniform
    pub fn set_vec4f32(&mut self, name: &str, vec: veclib::Vector4<f32>) {
        self.uniforms.insert(name.to_string(), Uniform::Vec4F32(vec));
    }
    // Set a vec4 i32 uniform
    pub fn set_vec4i32(&mut self, name: &str, vec: veclib::Vector4<i32>) {
        self.uniforms.insert(name.to_string(), Uniform::Vec4I32(vec));
    }
}

impl ShaderGPUObject {
    // Get the excecution group
    pub fn new_excecution_group(&self) -> ShaderUniformsGroup {
        ShaderUniformsGroup { uniforms: HashMap::new() }
    }
}

impl ComputeShaderGPUObject {
    // Get the excecution group
    pub fn new_uniform_group(&self) -> ShaderUniformsGroup {
        ShaderUniformsGroup { uniforms: HashMap::new() }
    }
}

impl ComputeShaderGPUObject {
    // Compute shader stuff you know
    pub fn run(&self, x: u16, y: u16, z: u16) {}
    pub fn lock_state(&self) {}
}

// Some identifiers that we will use to communicate from the Render Thread -> Main Thread
#[derive(Clone)]
pub enum GPUObject {
    None,                                  // This value was not initalized yet
    Model(ModelGPUObject),                 // The VAO ID
    SubShader(SubShaderGPUObject),         // The subshader program ID
    Shader(ShaderGPUObject),               // The shader program ID
    ComputeShader(ComputeShaderGPUObject), // Pretty much the same as a normal shader but we have some extra functions
    Texture(TextureGPUObject),             // The texture ID
}

impl Default for GPUObject {
    fn default() -> Self {
        Self::None
    }
}
