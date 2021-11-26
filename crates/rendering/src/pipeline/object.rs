use std::collections::HashMap;

use crate::{MaterialFlags, SubShaderType, TextureShaderAccessType, TextureType, Uniform};

// Cooler objects
#[derive(Clone, Default)]
pub struct ModelGPUObject {
    pub vertex_buf: u32,
    pub normal_buf: u32,
    pub uv_buf: u32,
    pub tangent_buf: u32,
    pub color_buf: u32,
    pub vertex_array_object: u32,
    pub element_buffer_object: u32,
    pub element_count: usize,
}

#[derive(Clone, Default)]
pub struct SubShaderGPUObject(pub SubShaderType, pub u32);
#[derive(Clone, Default)]
pub struct ShaderGPUObject(pub u32);
#[derive(Clone, Default)]
pub struct ComputeShaderGPUObject(pub u32);
#[derive(Clone, Copy, Default)]
pub struct TextureGPUObject(pub u32, pub (i32, u32, u32), pub TextureType);
#[derive(Clone, Default)]
pub struct CameraDataGPUObject {
    pub position: veclib::Vector3<f32>,
    pub rotation: veclib::Quaternion<f32>,
    pub clip_planes: veclib::Vector2<f32>,
    pub viewm: veclib::Matrix4x4<f32>,
    pub projm: veclib::Matrix4x4<f32>,
}
#[derive(Default)]
pub struct MaterialGPUObject(pub ShaderGPUObject, pub ShaderUniformsGroup, pub MaterialFlags);
#[derive(Default)]
pub struct RendererGPUObject(pub ModelGPUObject, pub MaterialGPUObject, pub veclib::Matrix4x4<f32>);

pub mod uniform_setters {
    use crate::{ShaderGPUObject, TextureGPUObject, TextureShaderAccessType};
    use std::ffi::CString;
    // Actually set the shader uniforms
    #[allow(temporary_cstring_as_ptr)]
    pub fn get_uniform_location(shader: u32, name: &str) -> i32 {
        unsafe {
            let x = gl::GetUniformLocation(shader, CString::new(name).unwrap().as_ptr());
            x
        }
    }
    // Set a f32 uniform
    pub unsafe fn set_f32(index: i32, value: &f32) {
        gl::Uniform1f(index, *value);
    }
    // Set a 2D image
    pub unsafe fn set_i2d(index: i32, texture: &TextureGPUObject, access_type: &TextureShaderAccessType) {
        // Converstion from wrapper to actual opengl values
        let new_access_type: u32;
        match access_type {
            TextureShaderAccessType::ReadOnly => new_access_type = gl::READ_ONLY,
            TextureShaderAccessType::WriteOnly => new_access_type = gl::WRITE_ONLY,
            TextureShaderAccessType::ReadWrite => new_access_type = gl::READ_WRITE,
        };
        let unit = index as u32;
        gl::BindTexture(gl::TEXTURE_2D, texture.0);
        gl::BindImageTexture(unit, texture.0, 0, gl::FALSE, 0, new_access_type, (texture.1).0 as u32);
    }
    // Set a i32
    pub unsafe fn set_i32(index: i32, value: &i32) {
        gl::Uniform1i(index, *value);
    }
    // Set a 3D image
    pub unsafe fn set_i3d(index: i32, texture: &TextureGPUObject, access_type: &TextureShaderAccessType) {
        // Converstion from wrapper to actual opengl values
        let new_access_type: u32;
        match access_type {
            TextureShaderAccessType::ReadOnly => new_access_type = gl::READ_ONLY,
            TextureShaderAccessType::WriteOnly => new_access_type = gl::WRITE_ONLY,
            TextureShaderAccessType::ReadWrite => new_access_type = gl::READ_WRITE,
        };
        let unit = index as u32;
        gl::BindTexture(gl::TEXTURE_3D, texture.0);
        gl::BindImageTexture(unit, texture.0, 0, gl::FALSE, 0, new_access_type, (texture.1).0 as u32);
    }
    // Set a matrix 4x4 f32
    pub unsafe fn set_mat44(index: i32, matrix: &veclib::Matrix4x4<f32>) {
        let ptr: *const f32 = &matrix[0];
        gl::UniformMatrix4fv(index, 1, gl::FALSE, ptr);
    }
    // Set a 1D texture
    pub unsafe fn set_t1d(index: i32, texture: &TextureGPUObject, active_texture_id: &u32) {
        gl::ActiveTexture(active_texture_id + 33984);
        gl::BindTexture(gl::TEXTURE_1D, texture.0);
        gl::Uniform1i(index, *active_texture_id as i32);
    }
    // Set a 2D texture
    pub unsafe fn set_t2d(index: i32, texture: &TextureGPUObject, active_texture_id: &u32) {
        gl::ActiveTexture(active_texture_id + 33984);
        gl::BindTexture(gl::TEXTURE_2D, texture.0);
        gl::Uniform1i(index, *active_texture_id as i32);
    }
    // Set a texture2d array
    pub unsafe fn set_t2da(index: i32, texture: &TextureGPUObject, active_texture_id: &u32) {
        gl::ActiveTexture(active_texture_id + 33984);
        gl::BindTexture(gl::TEXTURE_2D_ARRAY, texture.0);
        gl::Uniform1i(index, *active_texture_id as i32);
    }
    // Set a 3D texture
    pub unsafe fn set_t3d(index: i32, texture: &TextureGPUObject, active_texture_id: &u32) {
        gl::ActiveTexture(active_texture_id + 33984);
        gl::BindTexture(gl::TEXTURE_3D, texture.0);
        gl::Uniform1i(index, *active_texture_id as i32);
    }
    // Set a vec2 f32 uniform
    pub unsafe fn set_vec2f32(index: i32, vec: &veclib::Vector2<f32>) {
        gl::Uniform2f(index, vec[0], vec[1]);
    }
    // Set a vec2 i32 uniform
    pub unsafe fn set_vec2i32(index: i32, vec: &veclib::Vector2<i32>) {
        gl::Uniform2i(index, vec[0], vec[1]);
    }
    // Set a vec3 f32 uniform
    pub unsafe fn set_vec3f32(index: i32, vec: &veclib::Vector3<f32>) {
        gl::Uniform3f(index, vec[0], vec[1], vec[2]);
    }
    // Set a vec3 i32 uniform
    pub unsafe fn set_vec3i32(index: i32, vec: &veclib::Vector3<i32>) {
        gl::Uniform3i(index, vec[0], vec[1], vec[2]);
    }
    // Set a vec4 f32 uniform
    pub unsafe fn set_vec4f32(index: i32, vec: &veclib::Vector4<f32>) {
        gl::Uniform4f(index, vec[0], vec[1], vec[2], vec[3]);
    }
    // Set a vec4 i32 uniform
    pub unsafe fn set_vec4i32(index: i32, vec: &veclib::Vector4<i32>) {
        gl::Uniform4i(index, vec[0], vec[1], vec[2], vec[3]);
    }
}

// Run the shader uniforms of a specific shader
fn run_shader_uniform_group(shader: u32, group: &ShaderUniformsGroup) {
    // Set the default/custom uniforms
    use uniform_setters::*;
    for uniform in group.uniforms.iter() {
        let index = get_uniform_location(shader, uniform.0.as_str());
        unsafe {
            match &uniform.1 {
                Uniform::F32(x) => set_f32(index, x),
                Uniform::I32(x) => set_i32(index, x),
                Uniform::Vec2F32(x) => set_vec2f32(index, x),
                Uniform::Vec3F32(x) => set_vec3f32(index, x),
                Uniform::Vec4F32(x) => set_vec4f32(index, x),
                Uniform::Vec2I32(x) => set_vec2i32(index, x),
                Uniform::Vec3I32(x) => set_vec3i32(index, x),
                Uniform::Vec4I32(x) => set_vec4i32(index, x),
                Uniform::Mat44F32(x) => set_mat44(index, x),
                Uniform::Texture1D(x, y) => set_t1d(index, x, y),
                Uniform::Texture2D(x, y) => set_t2d(index, x, y),
                Uniform::Texture3D(x, y) => set_t3d(index, x, y),
                Uniform::Texture2DArray(x, y) => set_t2da(index, x, y),
                Uniform::Image2D(x, y) => set_i2d(index, x, y),
                Uniform::Image3D(x, y) => set_i3d(index, x, y),
                Uniform::Bool(x) => todo!(),
            }
        }
    }
}

// Each shader will contain a "shader excecution group" that will contain uniforms that must be sent to the GPU when that shader gets run
#[derive(Default, Clone)]
pub struct ShaderUniformsGroup {
    pub shader: u32,
    pub uniforms: HashMap<String, Uniform>,
}

// Gotta change the place where this shit is in
impl ShaderUniformsGroup {
    // Combine a shader uniform group with an another one
    pub fn combine(a: Self, b: Self, shader: u32) -> Self {
        let mut x = a.uniforms;
        let y = b.uniforms;
        for a in y {
            x.insert(a.0, a.1);
        }
        return Self { shader, uniforms: x };
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
    // Send this group data as a task to the render thread
    pub fn send(self) {
        // Actual send logic here
    }
    // Does the same thing as the "send" function above, but this time this assumes that we are already in the render thread
    // So we only need to consume the current uniform group and use the shader
    pub fn consume(self) {
        unsafe {
            gl::UseProgram(self.shader);
        }
        run_shader_uniform_group(self.shader, &self);
    }
}

impl ShaderGPUObject {
    // Get a new uniform group
    pub fn new_uniform_group(&self) -> ShaderUniformsGroup {
        ShaderUniformsGroup {
            shader: self.0,
            uniforms: HashMap::new(),
        }
    }
}

impl ComputeShaderGPUObject {
    // Get a new uniform group
    pub fn new_uniform_group(&self) -> ShaderUniformsGroup {
        ShaderUniformsGroup {
            shader: self.0,
            uniforms: HashMap::new(),
        }
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
    Renderer(usize),                       // The renderer ID
}

impl Default for GPUObject {
    fn default() -> Self {
        Self::None
    }
}
