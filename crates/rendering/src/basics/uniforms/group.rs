use std::{collections::HashMap, ffi::CString};

use veclib::{Vector, SupportedValue};

use crate::{Uniform, object::ObjectID, Texture, TextureShaderAccessType, Pipeline, ShaderUniformsSettings};


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
    // Set singular/multiple i32 value
    pub fn i32<T: SupportedValue + Vector<i32>>(&mut self, name: &str, val: T) {
        // Add the uniform
        self.uniforms.insert(name.to_string(), Uniform::I32(val.get_unsized()));
    }
    // Set singular/multiple f32 value
    pub fn f32<T: SupportedValue + Vector<f32>>(&mut self, name: &str, val: T) {
        // Add the uniform
        self.uniforms.insert(name.to_string(), Uniform::F32(val.get_unsized()));
    }
    // Set singular/multiple bool value
    pub fn bool<T: SupportedValue + Vector<T>>(&mut self, name: &str, val: T) {
        // Add the uniform
        self.uniforms.insert(name.to_string(), Uniform::Bool(val.get_unsized()));
    }
    // Set a "texture" uniform
    pub fn texture<T>(&mut self, name: &str, val: ObjectID<Texture>, active_texture_id: u32)
    {
        self.uniforms.push(Uniform::Texture(val, active_texture_id))
    }
    // Set a "image" uniform
    pub fn image(&mut self, val: ObjectID<Texture>, access_type: TextureShaderAccessType) {
        self.uniforms.push(Uniform::Image(val, access_type))
    }
    // Create self
    pub fn new() -> Self {
        Self { uniforms: HashMap::default() }
    }
    // Bind the shader and set the uniforms
    pub fn execute(&self, pipeline: &Pipeline, settings: ShaderUniformsSettings) -> Option<()> {
        // Get the shader program ID
        let shader =  pipeline.shaders.get(settings.shader_id.index)?;
        let program_id = shader.program;
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
                        veclib::UnsizedVector::Vec2(val) => set_bool(index, val),
                        veclib::UnsizedVector::Vec3(val) => set_bool(index, val),
                        veclib::UnsizedVector::Vec4(val) => set_bool(index, val),
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
                    Uniform::Texture(id, active_texture_id) => {
                        // We need to know the texture target first
                        let texture = pipeline.textures.get(id.index)?;
                        set_texture(texture, index, active_texture_id);
                    },
                    Uniform::Image(id, access_type) => {
                        // We need to know the texture target first
                        let texture = pipeline.textures.get(id.index)?;
                        set_image(texture, index, access_type);
                    },
                }
            }
        }
        Some(())
    }
}