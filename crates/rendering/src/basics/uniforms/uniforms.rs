use std::{collections::HashMap, ffi::CString};
use veclib::Vector;
use crate::{
    advanced::{
        atomic::{AtomicGroup, ClearCondition},
        shader_storage::ShaderStorage,
    },
    basics::{texture::{Texture, TextureAccessType}, shader::Shader},
    object::ObjectID,
    pipeline::Pipeline,
};
use super::{ShaderUniformsSettings, UniformsDefinitionMap, UniformError};

// Struct that allows us to set the uniforms for a specific shader
pub struct Uniforms<'a> {
    map: &'a UniformsDefinitionMap,
    pipeline: &'a Pipeline,
}

// Gotta change the place where this shit is in
impl<'a> Uniforms<'a> {
    
    // Get the location of a specific uniform
    
    pub fn set_i32(&self, name: &str, val: i32) -> Result<(), UniformError> {
        let location = self.map.get(name).ok_or(UniformError::invalid_location(name))?;
        unsafe {
            gl::Uniform1i(location, val);
        }
        Ok(())
    }
    
    pub fn set_vec2i32(&mut self, name: &str, vec2: veclib::Vector2<i32>) -> Result<(), UniformError> {
        let location = self.map.get(name).ok_or(UniformError::invalid_location(name))?;
        unsafe {
            gl::Uniform2i(location, vec2[0], vec2[1]);
        }
        Ok(())
    }
    
    pub fn set_vec3i32(&mut self, name: &str, vec3: veclib::Vector3<i32>) -> Result<(), UniformError> {
        let location = self.map.get(name)?;
        unsafe {
            gl::Uniform3i(location, vec3[0], vec3[1], vec3[2]);
        }
        Ok(())
    }
    
    pub fn set_f32(&mut self, name: &str, val: f32) -> Result<(), UniformError<'a>> {
        let location = self.map.get(name)?;
        unsafe {
            gl::Uniform1f(location, val);
        }
        Ok(())
    }
    
    pub fn set_vec2f32(&mut self, name: &str, vec2: veclib::Vector2<f32>) -> Result<(), UniformError> {
        let location = self.map.get(name)?;
        unsafe {
            gl::Uniform2f(location, vec2[0], vec2[1]);
        }
        Ok(())
    }
    
    pub fn set_vec3f32(&mut self, name: &str, vec3: veclib::Vector3<f32>) -> Result<(), UniformError> {
        let location = self.map.get(name)?;
        unsafe {
            gl::Uniform3f(location, vec3[0], vec3[1], vec3[3]);
        }
        Ok(())
    }
    
    pub fn set_bool(&mut self, name: &str, val: bool) -> Result<(), UniformError> {
        self.set_i32(name, val.into())
    }
    
    pub fn set_vec2bool(&mut self, name: &str, vec2: veclib::Vector2<bool>) -> Result<(), UniformError> {
        self.set_vec2i32(name, vec2.into())
    }
    
    pub fn set_vec3bool(&mut self, name: &str, vec3: veclib::Vector3<bool>) -> Result<(), UniformError> {
        self.set_vec3i32(name, vec3.into())
    }
    
    pub fn set_mat44f32(&mut self, name: &str, matrix: veclib::Matrix4x4<f32>) -> Result<(), UniformError> {
        let location = self.map.get(name)?;
        let ptr: *const f32 = &matrix[0];
        unsafe {
            gl::UniformMatrix4fv(location, 1, gl::FALSE, ptr);
        }
        Ok(())
    }
    
    pub fn set_texture(&mut self, name: &str, texture_id: ObjectID<Texture>, active_texture_id: u32) -> Result<(), UniformError> {
        let location = self.map.get(name)?;
        let texture = self.pipeline.textures.get(texture_id)?;
        unsafe {
            gl::ActiveTexture(active_texture_id + gl::TEXTURE0);
            gl::BindTexture(texture.target, texture.oid);
            gl::Uniform1i(location, *active_texture_id as i32);
        }
        Ok(())
    }
    
    pub fn set_image(&mut self, name: &str, texture_id: ObjectID<Texture>, access: TextureAccessType) -> Result<(), UniformError> {
        // Converstion from wrapper to actual OpenGL values
        let location = self.map.get(name)?;
        let texture = self.pipeline.textures.get(texture_id)?;
        let new_access_type: u32 = {
            if access.is_all() {
                gl::READ_WRITE
            } else if access.contains(TextureAccessType::READ) {
                gl::READ_ONLY
            } else if access.contains(TextureAccessType::WRITE) {
                gl::WRITE_ONLY
            } else {
                return Err(UniformError::new(name, "invalid texture access type"))
            }
        };
        unsafe {
            gl::BindTexture(texture.target, texture.oid);
            gl::BindImageTexture(location as u32, texture.oid, 0, gl::FALSE, 0, new_access_type, (texture.ifd).0 as u32);
        }
        Ok(())
    }
    pub fn set_atomic_group(&mut self, name: &str, atomic_group_id: ObjectID<AtomicGroup>, binding: u32) -> Result<(), UniformError> {
        let location = self.map.get(name)?;
        let atomic_group = self.pipeline.atomics.get(atomic_group_id)?;
        unsafe {
            gl::BindBuffer(gl::ATOMIC_COUNTER_BUFFER, atomic_group.oid);
            gl::BindBufferBase(gl::ATOMIC_COUNTER_BUFFER, *binding as u32, atomic_group.oid);
            gl::BindBuffer(gl::ATOMIC_COUNTER_BUFFER, 0);
        }
        Ok(())
    }
    // Set a shader storage uniform
    pub fn set_shader_storage(&mut self, name: &str, shader_storage_id: ObjectID<ShaderStorage>, binding: u32) -> Result<(), UniformError> {
        let location = self.map.get(name)?;
        let shader_storage = self.pipeline.shader_storages.get(shader_storage_id)?;
        unsafe {
            gl::BindBuffer(gl::SHADER_STORAGE_BUFFER, shader_storage.oid);
            gl::BindBufferBase(gl::SHADER_STORAGE_BUFFER, *binding as u32, shader_storage.oid);
            gl::BindBuffer(gl::SHADER_STORAGE_BUFFER, 0);
        }
        Ok(())
    }
}
