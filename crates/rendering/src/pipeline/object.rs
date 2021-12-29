use std::{collections::HashMap, ffi::CString};

use crate::{GPUObjectID, MaterialFlags, SubShaderType, TextureShaderAccessType, TextureType, Uniform};

use super::buffer::PipelineBuffer;

pub trait GPUObjectIdentifiable {
    // Get the GPU object ID of the current GPU object
    fn get_id(&self) -> GPUObjectID;
}

// Cooler objects
#[derive(Clone)]
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

#[derive(Clone)]
pub struct SubShaderGPUObject {
    pub subshader_type: SubShaderType,
    pub program: u32,
}

#[derive(Clone)]
pub struct ShaderGPUObject {
    pub program: u32,
}

#[derive(Clone)]
pub struct ComputeShaderGPUObject {
    pub program: u32,
}

#[derive(Clone, Copy)]
pub struct TextureGPUObject {
    pub texture_id: u32,
    pub ifd: (i32, u32, u32),
    pub ttype: TextureType,
}

#[derive(Clone)]
// TODO: Add this as an actual GPU object lel
pub struct CameraDataGPUObject {
    pub position: veclib::Vector3<f32>,
    pub rotation: veclib::Quaternion<f32>,
    pub clip_planes: veclib::Vector2<f32>,
    pub viewm: veclib::Matrix4x4<f32>,
    pub projm: veclib::Matrix4x4<f32>,
}

#[derive(Clone)]
pub struct MaterialGPUObject {
    pub shader: Option<GPUObjectID>,
    pub uniforms: GPUObjectID,
    pub flags: MaterialFlags,
}

#[derive(Clone)]
pub struct UniformsGPUObject {
    pub uniforms: ShaderUniformsGroup,
}

#[derive(Clone)]
pub struct RendererGPUObject {
    pub model_id: GPUObjectID,
    pub material_id: GPUObjectID,
    pub matrix: veclib::Matrix4x4<f32>,
    pub uniforms: Option<GPUObjectID>,
}

pub mod uniform_setters {
    use crate::{pipeline::buffer::PipelineBuffer, GPUObject, GPUObjectID, TextureGPUObject, TextureShaderAccessType};
    use std::ffi::CString;
    // Actually set the shader uniforms
    #[allow(temporary_cstring_as_ptr)]
    // Set a f32 uniform
    pub unsafe fn set_f32(index: i32, value: &f32) {
        gl::Uniform1f(index, *value);
    }
    // Set a 2D image
    pub unsafe fn set_i2d(pipeline_buffer: &PipelineBuffer, index: i32, id: &GPUObjectID, access_type: &TextureShaderAccessType) {
        let texture = pipeline_buffer.as_texture(id).unwrap();
        // Converstion from wrapper to actual opengl values
        let new_access_type: u32;
        match access_type {
            TextureShaderAccessType::ReadOnly => new_access_type = gl::READ_ONLY,
            TextureShaderAccessType::WriteOnly => new_access_type = gl::WRITE_ONLY,
            TextureShaderAccessType::ReadWrite => new_access_type = gl::READ_WRITE,
        };
        let unit = index as u32;
        gl::BindTexture(gl::TEXTURE_2D, texture.texture_id);
        gl::BindImageTexture(unit, texture.texture_id, 0, gl::FALSE, 0, new_access_type, (texture.ifd).0 as u32);
    }
    // Set a i32
    pub unsafe fn set_i32(index: i32, value: &i32) {
        gl::Uniform1i(index, *value);
    }
    // Set a 3D image
    pub unsafe fn set_i3d(pipeline_buffer: &PipelineBuffer, index: i32, id: &GPUObjectID, access_type: &TextureShaderAccessType) {
        let texture = pipeline_buffer.as_texture(id).unwrap();
        // Converstion from wrapper to actual opengl values
        let new_access_type: u32;
        match access_type {
            TextureShaderAccessType::ReadOnly => new_access_type = gl::READ_ONLY,
            TextureShaderAccessType::WriteOnly => new_access_type = gl::WRITE_ONLY,
            TextureShaderAccessType::ReadWrite => new_access_type = gl::READ_WRITE,
        };
        let unit = index as u32;
        gl::BindTexture(gl::TEXTURE_3D, texture.texture_id);
        gl::BindImageTexture(unit, texture.texture_id, 0, gl::FALSE, 0, new_access_type, (texture.ifd).0 as u32);
    }
    // Set a matrix 4x4 f32
    pub unsafe fn set_mat44(index: i32, matrix: &veclib::Matrix4x4<f32>) {
        let ptr: *const f32 = &matrix[0];
        gl::UniformMatrix4fv(index, 1, gl::FALSE, ptr);
    }
    // Set a 1D texture
    pub unsafe fn set_t1d(pipeline_buffer: &PipelineBuffer, index: i32, id: &GPUObjectID, active_texture_id: &u32) {
        let texture = pipeline_buffer.as_texture(id).unwrap();
        gl::ActiveTexture(active_texture_id + 33984);
        gl::BindTexture(gl::TEXTURE_1D, texture.texture_id);
        gl::Uniform1i(index, *active_texture_id as i32);
    }
    // Set a 2D texture
    pub unsafe fn set_t2d(pipeline_buffer: &PipelineBuffer, index: i32, id: &GPUObjectID, active_texture_id: &u32) {
        let texture = pipeline_buffer.as_texture(id).unwrap();
        gl::ActiveTexture(active_texture_id + 33984);
        gl::BindTexture(gl::TEXTURE_2D, texture.texture_id);
        gl::Uniform1i(index, *active_texture_id as i32);
    }
    // Set a texture2d array
    pub unsafe fn set_t2da(pipeline_buffer: &PipelineBuffer, index: i32, id: &GPUObjectID, active_texture_id: &u32) {
        let texture = pipeline_buffer.as_texture(id).unwrap();
        gl::ActiveTexture(active_texture_id + 33984);
        gl::BindTexture(gl::TEXTURE_2D_ARRAY, texture.texture_id);
        gl::Uniform1i(index, *active_texture_id as i32);
    }
    // Set a 3D texture
    pub unsafe fn set_t3d(pipeline_buffer: &PipelineBuffer, index: i32, id: &GPUObjectID, active_texture_id: &u32) {
        let texture = pipeline_buffer.as_texture(id).unwrap();
        gl::ActiveTexture(active_texture_id + 33984);
        gl::BindTexture(gl::TEXTURE_3D, texture.texture_id);
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
// Stores the current shader and the shader ID possibly of the shader linked to the uniforms
pub struct ShaderUniformsSettings {
    pub shader_id: Option<GPUObjectID>,
    pub shader_program_id: Option<u32>,
}

impl ShaderUniformsSettings {
    pub fn new_id(shader_id: &GPUObjectID) -> Self {
        Self {
            shader_id: Some(shader_id.clone()),
            shader_program_id: None,
        }
    } 
    pub fn new_program_id(shader: &ShaderGPUObject) -> Self {
        Self {
            shader_id: None,
            shader_program_id: Some(shader.program),
        }
    } 
}


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
        Self {
            uniforms: x,
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
        Self {
            uniforms: HashMap::default(),
        }
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
                    Uniform::Bool(_x) => todo!(),
                }
            }
        }
        Some(())
    }
}
// Some identifiers that we will use to communicate from the Render Thread -> Main Thread
#[derive(Clone)]
pub enum GPUObject {
    None, // This value was not initalized yet
    Model(ModelGPUObject),
    Material(MaterialGPUObject),
    Uniforms(UniformsGPUObject),
    SubShader(SubShaderGPUObject),
    Shader(ShaderGPUObject),
    ComputeShader(ComputeShaderGPUObject), // Pretty much the same as a normal shader but we have some extra functions
    Texture(TextureGPUObject),
    Renderer(RendererGPUObject),
}

impl Default for GPUObject {
    fn default() -> Self {
        Self::None
    }
}
