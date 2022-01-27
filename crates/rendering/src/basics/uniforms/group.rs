use std::{collections::HashMap, ffi::{CString, c_void}, sync::atomic::Ordering};
use veclib::Vector;

use crate::{
    basics::{texture::{Texture, TextureAccessType}, transfer::Transfer},
    object::ObjectID,
    pipeline::Pipeline, advanced::atomic::{AtomicCounter, ClearCondition},
};

use super::{ShaderUniformsSettings, Uniform};

// Each shader will contain a "shader excecution group" that will contain uniforms that must be sent to the GPU when that shader gets run
#[derive(Default, Clone)]
pub struct ShaderUniformsGroup {
    pub(crate) uniforms: HashMap<String, Uniform>,
}

// Gotta change the place where this shit is in
impl ShaderUniformsGroup {
    // Combine a shader uniform group with an another one
    pub fn combine(a: Self, b: Self) -> Self {
        let mut x = a.uniforms;
        let y = b.uniforms;
        for a in y {
            x.insert(a.0, a.1);
        }
        Self { uniforms: x }
    }
    // Set singular i32 value
    pub fn set_i32(&mut self, name: &str, val: i32) {
        self.uniforms.insert(name.to_string(), Uniform::I32(val.get_unsized()));
    }
    // Set a vector 2 of i32 values
    pub fn set_vec2i32(&mut self, name: &str, val: veclib::Vector2<i32>) {
        self.uniforms.insert(name.to_string(), Uniform::I32(val.get_unsized()));
    }
    // Set a vector 3 of i32 values
    pub fn set_vec3i32(&mut self, name: &str, val: veclib::Vector3<i32>) {
        self.uniforms.insert(name.to_string(), Uniform::I32(val.get_unsized()));
    }
    // Set singular f32 value
    pub fn set_f32(&mut self, name: &str, val: f32) {
        self.uniforms.insert(name.to_string(), Uniform::F32(val.get_unsized()));
    }
    // Set singular f64 value
    pub fn set_f64(&mut self, name: &str, val: f64) {
        self.uniforms.insert(name.to_string(), Uniform::F64(val.get_unsized()));
    }
    // Set a vector 2 of f32 values
    pub fn set_vec2f32(&mut self, name: &str, val: veclib::Vector2<f32>) {
        self.uniforms.insert(name.to_string(), Uniform::F32(val.get_unsized()));
    }
    // Set a vector 3 of f32 values
    pub fn set_vec3f32(&mut self, name: &str, val: veclib::Vector3<f32>) {
        self.uniforms.insert(name.to_string(), Uniform::F32(val.get_unsized()));
    }
    // Set singular bool value
    pub fn set_bool(&mut self, name: &str, val: bool) {
        self.uniforms.insert(name.to_string(), Uniform::Bool(val.get_unsized()));
    }
    // Set a vector 2 of bool values
    pub fn set_vec2bool(&mut self, name: &str, val: veclib::Vector2<bool>) {
        self.uniforms.insert(name.to_string(), Uniform::Bool(val.get_unsized()));
    }
    // Set a vector 3 of bool values
    pub fn set_vec3bool(&mut self, name: &str, val: veclib::Vector3<bool>) {
        self.uniforms.insert(name.to_string(), Uniform::Bool(val.get_unsized()));
    }
    // Set a matrix 4x4 of f32 values
    pub fn set_mat44f32(&mut self, name: &str, val: veclib::Matrix4x4<f32>) {
        self.uniforms.insert(name.to_string(), Uniform::Mat44F32(val));
    }
    // Set a "texture" uniform
    pub fn set_texture(&mut self, name: &str, val: ObjectID<Texture>, active_texture_id: u32) {
        self.uniforms.insert(name.to_string(), Uniform::Texture(val, active_texture_id));
    }
    // Set a "image" uniform
    pub fn set_image(&mut self, name: &str, val: ObjectID<Texture>, access: TextureAccessType) {
        self.uniforms.insert(name.to_string(), Uniform::Image(val, access));
    }
    // Set a "atomic counter" uniform
    pub fn set_atomic(&mut self, name: &str, val: Transfer<AtomicCounter>, binding: u32) {
        self.uniforms.insert(name.to_string(), Uniform::Counter(val, binding));
    }
    // Check if we have a specific uniform store
    pub fn contains_uniform(&self, name: &str) -> bool {
        self.uniforms.contains_key(name)
    }
    // Create self
    pub fn new() -> Self {
        Self { uniforms: HashMap::default() }
    }
    // Bind the shader and set the uniforms
    pub fn execute(&self, pipeline: &Pipeline, settings: ShaderUniformsSettings) -> Option<()> {
        // Get the shader program ID
        let program_id = settings.get_program_id(pipeline);
        unsafe {
            gl::UseProgram(program_id);
        }
        use super::setters::*;
        for (name, uniform) in self.uniforms.iter() {
            let index = unsafe { gl::GetUniformLocation(program_id, CString::new(name.clone()).ok()?.as_ptr()) };
            unsafe {
                match &uniform {
                    Uniform::Bool(unsized_vector) => match unsized_vector {
                        veclib::UnsizedVector::Single(val) => set_bool(index, val),
                        veclib::UnsizedVector::Vec2(val) => set_vec2bool(index, val),
                        veclib::UnsizedVector::Vec3(val) => set_vec3bool(index, val),
                        veclib::UnsizedVector::Vec4(val) => set_vec4bool(index, val),
                    },
                    Uniform::I32(unsized_vector) => match unsized_vector {
                        veclib::UnsizedVector::Single(val) => set_i32(index, val),
                        veclib::UnsizedVector::Vec2(val) => set_vec2i32(index, val),
                        veclib::UnsizedVector::Vec3(val) => set_vec3i32(index, val),
                        veclib::UnsizedVector::Vec4(val) => set_vec4i32(index, val),
                    },
                    Uniform::F32(unsized_vector) => match unsized_vector {
                        veclib::UnsizedVector::Single(val) => set_f32(index, val),
                        veclib::UnsizedVector::Vec2(val) => set_vec2f32(index, val),
                        veclib::UnsizedVector::Vec3(val) => set_vec3f32(index, val),
                        veclib::UnsizedVector::Vec4(val) => set_vec4f32(index, val),
                    },
                    Uniform::F64(unsized_vector) => match unsized_vector {
                        veclib::UnsizedVector::Single(val) => set_f64(index, val),
                        veclib::UnsizedVector::Vec2(val) => set_vec2f64(index, val),
                        veclib::UnsizedVector::Vec3(val) => set_vec3f64(index, val),
                        veclib::UnsizedVector::Vec4(val) => set_vec4f64(index, val),
                    },
                    Uniform::Mat44F32(matrix) => set_mat44f32(index, matrix),
                    Uniform::Texture(id, active_texture_id) => {
                        // We need to know the texture target first
                        let texture = pipeline.get_texture(*id)?;
                        set_texture(index, texture, active_texture_id);
                    }
                    Uniform::Image(id, access_type) => {
                        // We need to know the texture target first
                        let texture = pipeline.get_texture(*id)?;
                        set_image(index, texture, access_type);
                    }
                    Uniform::Counter(transfer, binding) => {
                        // If this is the first time we execute the shader that contains this atomic counter, we must initialize the counter as a valid buffer
                        pipeline.atomic_counter_create(transfer);
                        // Also clear the atomic counter before the shader execution if needed
                        match transfer.0.condition {
                            ClearCondition::BeforeShaderExecution => transfer.0.set(0),
                            _ => {},
                        }
                        set_atomic(index, &transfer.0, binding);
                    },
                }
            }
        }
        Some(())
    }
}
