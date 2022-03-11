use crate::{
    advanced::{atomic::AtomicGroup, raw::Buffer, shader_storage::ShaderStorage},
    basics::{
        shader::ShaderProgram,
        texture::{Texture},
    },
    pipeline::{Handle, Pipeline},
};

// Struct that allows us to set the uniforms for a specific shader
pub struct Uniforms<'a> {
    program: &'a ShaderProgram,
    pipeline: &'a Pipeline,
}

// Gotta change the place where this shit is in
impl<'a> Uniforms<'a> {
    // Create a uniforms setter using a shader program and the pipeline
    pub fn new(program: &'a ShaderProgram, pipeline: &'a Pipeline, autobind: bool) -> Self {
        let mut me = Self { program, pipeline };
        // Auto bind
        if autobind {
            me.bind();
        }
        me
    }
    // Get the location of a specific uniform using it's name, and returns an error if it could not
    fn get_location(&self, name: &str) -> i32 {
        //if res == -1 { eprintln!("{} does not have a valid uniform location for program {}", name, self.program); }
        self.program.mappings().get(name).cloned().unwrap_or(-1)
    }
    // Bind the shader for execution/rendering
    pub fn bind(&mut self) {
        unsafe { gl::UseProgram(self.program.program()) }
        // Set some global uniforms while we're at it
        self.set_f32("_time", self.pipeline.time().elapsed as f32);
        self.set_f32("_delta", self.pipeline.time().delta as f32);
        self.set_vec2i32("_resolution", self.pipeline.window.dimensions.into());
    }
    // U32
    pub fn set_u32(&mut self, name: &str, val: u32) {
        let location = self.get_location(name);
        if location == -1 {
            return;
        }
        unsafe {
            gl::Uniform1ui(location, val);
        }
    }
    pub fn set_vec2u32(&mut self, name: &str, vec2: veclib::Vector2<u32>) {
        let location = self.get_location(name);
        if location == -1 {
            return;
        }
        unsafe {
            gl::Uniform2ui(location, vec2[0], vec2[1]);
        }
    }
    pub fn set_vec3u32(&mut self, name: &str, vec3: veclib::Vector3<u32>) {
        let location = self.get_location(name);
        if location == -1 {
            return;
        }
        unsafe {
            gl::Uniform3ui(location, vec3[0], vec3[1], vec3[2]);
        }
    }
    // I32
    pub fn set_i32(&mut self, name: &str, val: i32) {
        let location = self.get_location(name);
        if location == -1 {
            return;
        }
        unsafe {
            gl::Uniform1i(location, val);
        }
    }
    pub fn set_vec2i32(&mut self, name: &str, vec2: veclib::Vector2<i32>) {
        let location = self.get_location(name);
        if location == -1 {
            return;
        }
        unsafe {
            gl::Uniform2i(location, vec2[0], vec2[1]);
        }
    }
    pub fn set_vec3i32(&mut self, name: &str, vec3: veclib::Vector3<i32>) {
        let location = self.get_location(name);
        if location == -1 {
            return;
        }
        unsafe {
            gl::Uniform3i(location, vec3[0], vec3[1], vec3[2]);
        }
    }
    // F32
    pub fn set_f32(&mut self, name: &str, val: f32) {
        let location = self.get_location(name);
        if location == -1 {
            return;
        }
        unsafe {
            gl::Uniform1f(location, val);
        }
    }
    pub fn set_vec2f32(&mut self, name: &str, vec2: veclib::Vector2<f32>) {
        let location = self.get_location(name);
        if location == -1 {
            return;
        }
        unsafe {
            gl::Uniform2f(location, vec2[0], vec2[1]);
        }
    }
    pub fn set_vec3f32(&mut self, name: &str, vec3: veclib::Vector3<f32>) {
        let location = self.get_location(name);
        if location == -1 {
            return;
        }
        unsafe {
            gl::Uniform3f(location, vec3[0], vec3[1], vec3[2]);
        }
    }
    // Bool
    pub fn set_bool(&mut self, name: &str, val: bool) {
        self.set_i32(name, val.into());
    }
    pub fn set_vec2bool(&mut self, name: &str, vec2: veclib::Vector2<bool>) {
        self.set_vec2i32(name, vec2.into());
    }
    pub fn set_vec3bool(&mut self, name: &str, vec3: veclib::Vector3<bool>) {
        self.set_vec3i32(name, vec3.into());
    }
    // Textures & others
    pub fn set_mat44f32(&mut self, name: &str, matrix: &veclib::Matrix4x4<f32>) {
        let location = self.get_location(name);
        if location == -1 {
            return;
        }
        let ptr: *const f32 = &matrix[0];
        unsafe {
            gl::UniformMatrix4fv(location, 1, gl::FALSE, ptr);
        }
    }
    pub fn set_texture(&mut self, name: &str, texture: &Handle<Texture>) {
        let location = self.get_location(name);
        if location == -1 {
            return;
        }
        let texture = self.pipeline.textures.get(texture).unwrap();
        // Get the active texture ID from the program
        let mut used_texture_units = self.program.used_texture_units().borrow_mut(); 
        if !used_texture_units.contains_key(name) {
            // Never existed before, add it
            let len = used_texture_units.len();
            used_texture_units.insert(name.to_string(), len);
        }

        // Set
        if let Some(&offset) = used_texture_units.get(name) {
            unsafe {
                gl::ActiveTexture(offset as u32 + gl::TEXTURE0);
                gl::BindTexture(texture.target(), texture.buffer());
                gl::Uniform1i(location, offset as i32);
            }
        }
    }
    pub fn set_atomic_group(&mut self, _name: &str, atomic: &mut AtomicGroup, binding: u32) {
        unsafe {
            gl::BindBuffer(gl::ATOMIC_COUNTER_BUFFER, atomic.storage().storage().buffer());
            gl::BindBufferBase(gl::ATOMIC_COUNTER_BUFFER, binding, atomic.storage().storage().buffer());
            gl::BindBuffer(gl::ATOMIC_COUNTER_BUFFER, 0);
        }
    }
    pub fn set_shader_storage<Buffer: crate::advanced::raw::Buffer>(&mut self, _name: &str, shader_storage: &mut ShaderStorage<Buffer>, binding: u32) {
        unsafe {
            gl::BindBuffer(gl::SHADER_STORAGE_BUFFER, shader_storage.storage().storage().buffer());
            gl::BindBufferBase(gl::SHADER_STORAGE_BUFFER, binding, shader_storage.storage().storage().buffer());
            gl::BindBuffer(gl::SHADER_STORAGE_BUFFER, 0);
        }
    }
}
