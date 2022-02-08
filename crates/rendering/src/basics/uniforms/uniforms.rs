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
    // Create a new uniforms setter using some shaderuniformssettings and a pipeline
    pub(crate) fn new(settings: &'a ShaderUniformsSettings, pipeline: &'a Pipeline) -> Self {
        let program = settings._type.get_program(pipeline);
        let map = pipeline.cached.uniform_definitions.get(&program).unwrap();
        Self { map, program, pipeline }
    }
    // Create some new uniforms using a mutable pipeline
    // This should only be accessed by the EoF external callbacks
    // This automatically binds the shader as well
    pub fn using_mut_pipeline(settings: &'a ShaderUniformsSettings, pipeline: &'a mut Pipeline) -> Self {
        let program = settings._type.get_program(pipeline);
        let map = pipeline.cached.uniform_definitions.get(&program).unwrap();
        let uniforms = Self { map, program, pipeline };
        uniforms.bind_shader();
        uniforms
    }
    // Get the location of a specific uniform using it's name, and returns an error if it could not
    fn get_location(&self, name: &str) -> i32 {
        let res = self.map.get(name).unwrap_or(-1);
        //if res == -1 { eprintln!("{} does not have a valid uniform location for program {}", name, self.program); }
        res
    }
    // Bind the shader for execution/rendering
    pub(crate) fn bind_shader(&self) {
        unsafe { gl::UseProgram(self.program) }
        // Set some global uniforms while we're at it
        self.set_f32("_time", self.pipeline.time.0 as f32);
        self.set_f32("_delta", self.pipeline.time.1 as f32);
        self.set_vec2i32("_resolution", self.pipeline.window.dimensions.into());
    }

    // I32
    pub fn set_i32(&self, name: &str, val: i32) {
        let location = self.get_location(name);
        if location == -1 {
            return;
        }
        unsafe {
            gl::Uniform1i(location, val);
        }
    }
    pub fn set_vec2i32(&self, name: &str, vec2: veclib::Vector2<i32>) {
        let location = self.get_location(name);
        if location == -1 {
            return;
        }
        unsafe {
            gl::Uniform2i(location, vec2[0], vec2[1]);
        }
    }
    pub fn set_vec3i32(&self, name: &str, vec3: veclib::Vector3<i32>) {
        let location = self.get_location(name);
        if location == -1 {
            return;
        }
        unsafe {
            gl::Uniform3i(location, vec3[0], vec3[1], vec3[2]);
        }
    }
    // F32
    pub fn set_f32(&self, name: &str, val: f32) {
        let location = self.get_location(name);
        if location == -1 {
            return;
        }
        unsafe {
            gl::Uniform1f(location, val);
        }
    }
    pub fn set_vec2f32(&self, name: &str, vec2: veclib::Vector2<f32>) {
        let location = self.get_location(name);
        if location == -1 {
            return;
        }
        unsafe {
            gl::Uniform2f(location, vec2[0], vec2[1]);
        }
    }
    pub fn set_vec3f32(&self, name: &str, vec3: veclib::Vector3<f32>) {
        let location = self.get_location(name);
        if location == -1 {
            return;
        }
        unsafe {
            gl::Uniform3f(location, vec3[0], vec3[1], vec3[2]);
        }
    }
    // Bool
    pub fn set_bool(&self, name: &str, val: bool) {
        self.set_i32(name, val.into());
    }
    pub fn set_vec2bool(&self, name: &str, vec2: veclib::Vector2<bool>) {
        self.set_vec2i32(name, vec2.into());
    }
    pub fn set_vec3bool(&self, name: &str, vec3: veclib::Vector3<bool>) {
        self.set_vec3i32(name, vec3.into());
    }
    // Textures & others
    pub fn set_mat44f32(&self, name: &str, matrix: veclib::Matrix4x4<f32>) {
        let location = self.get_location(name);
        if location == -1 {
            return;
        }
        let ptr: *const f32 = &matrix[0];
        unsafe {
            gl::UniformMatrix4fv(location, 1, gl::FALSE, ptr);
        }
    }
    pub fn set_texture(&self, name: &str, texture_id: ObjectID<Texture>, active_texture_id: u32) {
        let location = self.get_location(name);
        if location == -1 {
            return;
        }
        let texture = if let Some(x) = self.pipeline.textures.get(texture_id) {
            x
        } else {
            return;
        };
        unsafe {
            gl::ActiveTexture(active_texture_id + gl::TEXTURE0);
            gl::BindTexture(texture.target, texture.oid);
            gl::Uniform1i(location, active_texture_id as i32);
        }
    }
    pub fn set_image(&self, name: &str, texture_id: ObjectID<Texture>, access: TextureAccessType) {
        let location = self.get_location(name);
        if location == -1 {
            return;
        }
        // Converstion from wrapper to actual OpenGL values
        let texture = if let Some(x) = self.pipeline.textures.get(texture_id) {
            x
        } else {
            return;
        };
        let new_access_type: u32 = {
            if access.is_all() {
                gl::READ_WRITE
            } else if access.contains(TextureAccessType::READ) {
                gl::READ_ONLY
            } else if access.contains(TextureAccessType::WRITE) {
                gl::WRITE_ONLY
            } else {
                panic!()
            }
        };
        unsafe {
            gl::BindTexture(texture.target, texture.oid);
            gl::BindImageTexture(location as u32, texture.oid, 0, gl::FALSE, 0, new_access_type, (texture.ifd).0 as u32);
        }
    }
    pub fn set_atomic_group(&self, name: &str, atomic_group_id: ObjectID<AtomicGroup>, binding: u32) {
        let atomic_group = if let Some(x) = self.pipeline.atomics.get(atomic_group_id) {
            x
        } else {
            return;
        };
        unsafe {
            gl::BindBuffer(gl::ATOMIC_COUNTER_BUFFER, atomic_group.oid);
            gl::BindBufferBase(gl::ATOMIC_COUNTER_BUFFER, binding, atomic_group.oid);
            gl::BindBuffer(gl::ATOMIC_COUNTER_BUFFER, 0);
        }
    }
    pub fn set_shader_storage(&self, name: &str, shader_storage_id: ObjectID<ShaderStorage>, binding: u32) {
        let shader_storage = if let Some(x) = self.pipeline.shader_storages.get(shader_storage_id) {
            x
        } else {
            return;
        };
        unsafe {
            gl::BindBuffer(gl::SHADER_STORAGE_BUFFER, shader_storage.oid);
            gl::BindBufferBase(gl::SHADER_STORAGE_BUFFER, binding, shader_storage.oid);
            gl::BindBuffer(gl::SHADER_STORAGE_BUFFER, 0);
        }
    }
}
