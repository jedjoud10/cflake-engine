use std::collections::HashMap;
use crate::Uniform;

// Each shader will contain a "shader excecution group" that will contain uniforms that must be sent to the GPU when that shader gets run
#[derive(Clone)]
pub struct ShaderUniformsGroup {
    pub uniforms: HashMap<String, Uniform>,
}

// Gotta change the place where this shit is in
impl ShaderUniformsGroup {
    // Combine a shader uniform group with an another one
    pub fn combine(a: &Self, b: &Self) -> Self {
        let mut x = a.uniforms.clone();
        let y = b.uniforms.clone();
        for a in y {
            x.insert(a.0, a.1);
        }
        Self { uniforms: x }
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
    pub fn set_i2d(&mut self, name: &str, texture: &GPUObjectID, access_type: TextureShaderAccessType) {
        self.uniforms.insert(name.to_string(), Uniform::Image2D(texture.clone(), access_type));
    }
    // Set a i32
    pub fn set_i32(&mut self, name: &str, value: i32) {
        self.uniforms.insert(name.to_string(), Uniform::I32(value));
    }
    // Set a 3D image
    pub fn set_i3d(&mut self, name: &str, texture: &GPUObjectID, access_type: TextureShaderAccessType) {
        self.uniforms.insert(name.to_string(), Uniform::Image3D(texture.clone(), access_type));
    }
    // Set a matrix 4x4 f32
    pub fn set_mat44(&mut self, name: &str, matrix: veclib::Matrix4x4<f32>) {
        self.uniforms.insert(name.to_string(), Uniform::Mat44F32(matrix));
    }
    // Set a 1D texture
    pub fn set_t1d(&mut self, name: &str, texture: &GPUObjectID, active_texture_id: u32) {
        self.uniforms.insert(name.to_string(), Uniform::Texture1D(texture.clone(), active_texture_id));
    }
    // Set a 2D texture
    pub fn set_t2d(&mut self, name: &str, texture: &GPUObjectID, active_texture_id: u32) {
        self.uniforms.insert(name.to_string(), Uniform::Texture2D(texture.clone(), active_texture_id));
    }
    // Set a texture2d array
    pub fn set_t2da(&mut self, name: &str, texture: &GPUObjectID, active_texture_id: u32) {
        self.uniforms.insert(name.to_string(), Uniform::Texture2DArray(texture.clone(), active_texture_id));
    }
    // Set a 3D texture
    pub fn set_t3d(&mut self, name: &str, texture: &GPUObjectID, active_texture_id: u32) {
        self.uniforms.insert(name.to_string(), Uniform::Texture3D(texture.clone(), active_texture_id));
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
    // Create self
    pub fn new() -> Self {
        Self { uniforms: HashMap::default() }
    }
    // Bind the shader and set the uniforms
    pub fn execute(&self, buf: &PipelineBuffer, settings: ShaderUniformsSettings) -> Option<()> {
        // Get the shader program ID
        let program_id = match settings.shader_program_id {
            Some(x) => x,
            None => match settings.shader_id {
                Some(id) => {
                    // Might be either a compute or a normal shader
                    if let Option::Some(x) = buf.as_shader(&id) {
                        x.program
                    } else if let Option::Some(x) = buf.as_compute_shader(&id) {
                        x.program
                    } else {
                        return None;
                    }
                }
                None => {
                    return None;
                }
            },
        };
        unsafe {
            gl::UseProgram(program_id);
        }
        use super::uniform_setters::*;
        for (name, uniform) in self.uniforms.iter() {
            let index = unsafe { gl::GetUniformLocation(program_id, CString::new(name.clone()).ok()?.as_ptr()) };
            unsafe {
                match &uniform {
                    Uniform::F32(x) => set_f32(index, x),
                    Uniform::I32(x) => set_i32(index, x),
                    Uniform::Vec2F32(x) => set_vec2f32(index, x),
                    Uniform::Vec3F32(x) => set_vec3f32(index, x),
                    Uniform::Vec4F32(x) => set_vec4f32(index, x),
                    Uniform::Vec2I32(x) => set_vec2i32(index, x),
                    Uniform::Vec3I32(x) => set_vec3i32(index, x),
                    Uniform::Vec4I32(x) => set_vec4i32(index, x),
                    Uniform::Mat44F32(x) => set_mat44(index, x),
                    Uniform::Texture1D(x, y) => set_t1d(buf, index, x, y),
                    Uniform::Texture2D(x, y) => set_t2d(buf, index, x, y),
                    Uniform::Texture3D(x, y) => set_t3d(buf, index, x, y),
                    Uniform::Texture2DArray(x, y) => set_t2da(buf, index, x, y),
                    Uniform::Image2D(x, y) => set_i2d(buf, index, x, y),
                    Uniform::Image3D(x, y) => set_i3d(buf, index, x, y),
                    Uniform::Bool(x) => set_bool(index, x),
                }
            }
        }
        Some(())
    }
}