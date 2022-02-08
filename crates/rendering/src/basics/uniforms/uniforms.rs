use super::{ShaderUniformsSettings, UniformError, UniformsDefinitionMap};
use crate::{
    advanced::{
        atomic::{AtomicGroup, ClearCondition},
        shader_storage::ShaderStorage,
    },
    basics::{
        shader::Shader,
        texture::{Texture, TextureAccessType},
    },
    object::ObjectID,
    pipeline::Pipeline,
};
use std::{collections::HashMap, ffi::CString};
use veclib::Vector;

// Struct that allows us to set the uniforms for a specific shader
pub struct Uniforms<'a> {
    map: &'a UniformsDefinitionMap,
    program: u32,
    pipeline: &'a Pipeline,
}

impl<'a> !Sync for Uniforms<'a> {}

// Gotta change the place where this shit is in
impl<'a> Uniforms<'a> {
    // Create a new uniforms setter using a shader uniforms settings and a pipeline
    // (we must only call this internally whenever we call the set uniform callback)
    pub(crate) fn new(settings: &'a ShaderUniformsSettings, pipeline: &'a Pipeline) -> Self {
        let program = settings._type.get_program(pipeline);
        let map = pipeline.cached.uniform_defitions.get(&program).unwrap();
        Self { map, program, pipeline }
    }
    // Get the location of a specific uniform using it's name, and returns an error if it could not
    fn get_location(&self, name: &str) -> Result<i32, UniformError> {
        self.map.get(name).map_or(Err(UniformError::invalid_location(name)), |x| Ok(x))
    }
    // Bind the shader for execution/rendering
    pub(crate) fn bind_shader(&self) {
        unsafe { gl::UseProgram(self.program) }
    }

    // I32
    pub fn set_i32(&self, name: &str, val: i32) -> Result<(), UniformError> {
        let location = self.get_location(name)?;
        unsafe {
            gl::Uniform1i(location, val);
        }
        Ok(())
    }
    pub fn set_vec2i32(&self, name: &str, vec2: veclib::Vector2<i32>) -> Result<(), UniformError> {
        let location = self.get_location(name)?;
        unsafe {
            gl::Uniform2i(location, vec2[0], vec2[1]);
        }
        Ok(())
    }
    pub fn set_vec3i32(&self, name: &str, vec3: veclib::Vector3<i32>) -> Result<(), UniformError> {
        let location = self.get_location(name)?;
        unsafe {
            gl::Uniform3i(location, vec3[0], vec3[1], vec3[2]);
        }
        Ok(())
    }
    // F32
    pub fn set_f32(&self, name: &str, val: f32) -> Result<(), UniformError> {
        let location = self.get_location(name)?;
        unsafe {
            gl::Uniform1f(location, val);
        }
        Ok(())
    }
    pub fn set_vec2f32(&self, name: &str, vec2: veclib::Vector2<f32>) -> Result<(), UniformError> {
        let location = self.get_location(name)?;
        unsafe {
            gl::Uniform2f(location, vec2[0], vec2[1]);
        }
        Ok(())
    }
    pub fn set_vec3f32(&self, name: &str, vec3: veclib::Vector3<f32>) -> Result<(), UniformError> {
        let location = self.get_location(name)?;
        unsafe {
            gl::Uniform3f(location, vec3[0], vec3[1], vec3[3]);
        }
        Ok(())
    }
    // Bool
    pub fn set_bool(&self, name: &str, val: bool) -> Result<(), UniformError> {
        self.set_i32(name, val.into())
    }
    pub fn set_vec2bool(&self, name: &str, vec2: veclib::Vector2<bool>) -> Result<(), UniformError> {
        self.set_vec2i32(name, vec2.into())
    }
    pub fn set_vec3bool(&self, name: &str, vec3: veclib::Vector3<bool>) -> Result<(), UniformError> {
        self.set_vec3i32(name, vec3.into())
    }
    // Textures & others
    pub fn set_mat44f32(&self, name: &str, matrix: veclib::Matrix4x4<f32>) -> Result<(), UniformError> {
        let location = self.get_location(name)?;
        let ptr: *const f32 = &matrix[0];
        unsafe {
            gl::UniformMatrix4fv(location, 1, gl::FALSE, ptr);
        }
        Ok(())
    }
    pub fn set_texture(&self, name: &str, texture_id: ObjectID<Texture>, active_texture_id: u32) -> Result<(), UniformError> {
        let location = self.get_location(name)?;
        let texture = self.pipeline.textures.get(texture_id).ok_or(UniformError::new(name, "invalid texture id"))?;
        unsafe {
            gl::ActiveTexture(active_texture_id + gl::TEXTURE0);
            gl::BindTexture(texture.target, texture.oid);
            gl::Uniform1i(location, active_texture_id as i32);
        }
        Ok(())
    }
    pub fn set_image(&self, name: &str, texture_id: ObjectID<Texture>, access: TextureAccessType) -> Result<(), UniformError> {
        let location = self.get_location(name)?;
        // Converstion from wrapper to actual OpenGL values
        let texture = self.pipeline.textures.get(texture_id).ok_or(UniformError::new(name, "invalid texture id"))?;
        let new_access_type: u32 = {
            if access.is_all() {
                gl::READ_WRITE
            } else if access.contains(TextureAccessType::READ) {
                gl::READ_ONLY
            } else if access.contains(TextureAccessType::WRITE) {
                gl::WRITE_ONLY
            } else {
                return Err(UniformError::new(name, "invalid texture access type"));
            }
        };
        unsafe {
            gl::BindTexture(texture.target, texture.oid);
            gl::BindImageTexture(location as u32, texture.oid, 0, gl::FALSE, 0, new_access_type, (texture.ifd).0 as u32);
        }
        Ok(())
    }
    pub fn set_atomic_group(&self, name: &str, atomic_group_id: ObjectID<AtomicGroup>, binding: u32) -> Result<(), UniformError> {
        let location = self.get_location(name)?;
        let atomic_group = self.pipeline.atomics.get(atomic_group_id).ok_or(UniformError::new(name, "invalid atomic group id"))?;
        unsafe {
            gl::BindBuffer(gl::ATOMIC_COUNTER_BUFFER, atomic_group.oid);
            gl::BindBufferBase(gl::ATOMIC_COUNTER_BUFFER, binding, atomic_group.oid);
            gl::BindBuffer(gl::ATOMIC_COUNTER_BUFFER, 0);
        }
        Ok(())
    }
    pub fn set_shader_storage(&self, name: &str, shader_storage_id: ObjectID<ShaderStorage>, binding: u32) -> Result<(), UniformError> {
        let location = self.get_location(name)?;
        let shader_storage = self
            .pipeline
            .shader_storages
            .get(shader_storage_id)
            .ok_or(UniformError::new(name, "invalid shader storage id"))?;
        unsafe {
            gl::BindBuffer(gl::SHADER_STORAGE_BUFFER, shader_storage.oid);
            gl::BindBufferBase(gl::SHADER_STORAGE_BUFFER, binding, shader_storage.oid);
            gl::BindBuffer(gl::SHADER_STORAGE_BUFFER, 0);
        }
        Ok(())
    }
}
