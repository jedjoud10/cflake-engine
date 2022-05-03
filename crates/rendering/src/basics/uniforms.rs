use gl::types::GLuint;

use crate::{
    basics::{
        shader::Program,
        texture::{BundledTexture2D, Texture, Texture2D, CubeMap},
    },
    pipeline::{Handle, Pipeline}, advanced::buffer::Buffer,
};
// Bound uniform object that can set OpenGL uniforms
pub struct Uniforms<'a> {
    program: &'a Program,
    pipeline: &'a Pipeline,
}

impl<'a> Uniforms<'a> {
    // Create a uniforms setter using a shader that can get it's program fetched and the pipeline
    pub fn new(program: &Option<Program>, pipeline: &Pipeline, closure: impl FnOnce(Uniforms)) {
        // Uh-oh... invalid program
        let program = program.as_ref().expect("Invalid program given, the shader given was not properly initialized");

        // Bind the OpenGL shader
        unsafe { gl::UseProgram(program.name()) }
        let mut uniforms = Uniforms { program, pipeline };

        // Set the general snippet uniforms while we're at it
        uniforms.set_f32("_time", pipeline.elapsed());
        uniforms.set_f32("_delta", pipeline.delta());
        uniforms.set_vec2i32("_resolution", pipeline.window().dimensions().as_().into());
        
        // Set the camera snippet uniforms
        let camera = pipeline.camera();
        uniforms.set_mat44f32("_pv_matrix", &camera.proj_view);
        uniforms.set_vec2f32("_nf_planes", camera.clips);
        uniforms.set_vec3f32("_cam_pos", camera.position);
        uniforms.set_vec3f32("_cam_dir", camera.forward);  
        
        closure(uniforms);
    }
    // Get the location of a specific uniform using it's name, and returns an error if it could not
    fn get_location(&self, name: &str) -> i32 {
        //if res == -1 { eprintln!("{} does not have a valid uniform location for program {}", name, self.program); }
        self.program.mappings().get(name).cloned().unwrap_or(-1)
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
    pub fn set_vec2u32(&mut self, name: &str, vec2: vek::Vec2<u32>) {
        let location = self.get_location(name);
        if location == -1 {
            return;
        }
        unsafe {
            gl::Uniform2ui(location, vec2[0], vec2[1]);
        }
    }
    pub fn set_vec3u32(&mut self, name: &str, vec3: vek::Vec3<u32>) {
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
    pub fn set_vec2i32(&mut self, name: &str, vec2: vek::Vec2<i32>) {
        let location = self.get_location(name);
        if location == -1 {
            return;
        }
        unsafe {
            gl::Uniform2i(location, vec2[0], vec2[1]);
        }
    }
    pub fn set_vec3i32(&mut self, name: &str, vec3: vek::Vec3<i32>) {
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
    pub fn set_vec2f32(&mut self, name: &str, vec2: vek::Vec2<f32>) {
        let location = self.get_location(name);
        if location == -1 {
            return;
        }
        unsafe {
            gl::Uniform2f(location, vec2[0], vec2[1]);
        }
    }
    pub fn set_vec3f32(&mut self, name: &str, vec3: vek::Vec3<f32>) {
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
    pub fn set_vec2bool(&mut self, name: &str, vec2: vek::Vec2<bool>) {
        self.set_vec2i32(name, vec2.as_());
    }
    pub fn set_vec3bool(&mut self, name: &str, vec3: vek::Vec3<bool>) {
        self.set_vec3i32(name, vec3.as_());
    }
    // Textures & others
    pub fn set_mat44f32(&mut self, name: &str, matrix: &vek::Mat4<f32>) {
        let location = self.get_location(name);
        if location == -1 {
            return;
        }
        unsafe {
            gl::UniformMatrix4fv(location, 1, gl::FALSE, matrix.as_col_ptr());
        }
    }
    // Bind a texture to a specific location using it's uniform name
    pub fn set_texture<Tex: Texture>(&mut self, name: &str, texture: &Handle<Tex>) {
        // Return early if the location is invalid
        let location = self.get_location(name);
        if location == -1 {
            return;
        }

        // Get the active texture ID from the program
        let mut used_texture_units = self.program.used_texture_units().borrow_mut();
        if !used_texture_units.contains_key(name) {
            // Never existed before, add it
            let len = used_texture_units.len();
            used_texture_units.insert(name.to_string(), len);
        }

        // Bind the texture into the valid texture unity
        if let Some(&offset) = used_texture_units.get(name) {
            // Fetch the texture using it's handle
            let texture = self.pipeline.get(texture).unwrap();

            unsafe {
                gl::ActiveTexture(offset as u32 + gl::TEXTURE0);
                gl::BindTexture(texture.target().unwrap(), texture.name().unwrap());
                gl::Uniform1i(location, offset as i32);
            }
        }
    }

    // Bind a buffer to a specific binding point, without the name
    pub fn set_buffer<T: Copy>(&mut self, buffer: &mut Buffer<T>, binding: u32) {
        unsafe {
            gl::BindBuffer(buffer.target(), buffer.name());
            gl::BindBufferBase(buffer.target(), binding, buffer.name());
        }
    }
}
