use std::mem::size_of;

use crate::{
    advanced::{atomic::AtomicGroup, shaderstorage::ShaderStorage},
    basics::texture::{Texture, TextureAccessType},
};

// Actually set the shader uniforms
#[allow(temporary_cstring_as_ptr)]
// Set a f32 uniform
pub unsafe fn set_f32(index: i32, value: &f32) {
    gl::Uniform1f(index, *value);
}
// Set a f64 uniform
pub unsafe fn set_f64(index: i32, value: &f64) {
    gl::Uniform1d(index, *value);
}
// Set an image that can be modified inside the shader
pub unsafe fn set_image(index: i32, texture: &Texture, access_type: &TextureAccessType) {
    // Converstion from wrapper to actual opengl values
    let new_access_type: u32 = {
        if access_type.is_all() {
            gl::READ_WRITE
        } else if access_type.contains(TextureAccessType::READ) {
            gl::READ_ONLY
        } else if access_type.contains(TextureAccessType::WRITE) {
            gl::WRITE_ONLY
        } else {
            panic!()
        }
    };
    let unit = index as u32;
    gl::BindTexture(texture.target, texture.oid);
    gl::BindImageTexture(unit, texture.oid, 0, gl::FALSE, 0, new_access_type, (texture.ifd).0 as u32);
}
// Set a i32
pub unsafe fn set_i32(index: i32, value: &i32) {
    gl::Uniform1i(index, *value);
}
// Set a matrix 4x4 f32
pub unsafe fn set_mat44f32(index: i32, matrix: &veclib::Matrix4x4<f32>) {
    let ptr: *const f32 = &matrix[0];
    gl::UniformMatrix4fv(index, 1, gl::FALSE, ptr);
}
// Set a texture
pub unsafe fn set_texture(index: i32, texture: &Texture, active_texture_id: &u32) {
    gl::ActiveTexture(active_texture_id + 33984);
    gl::BindTexture(texture.target, texture.oid);
    gl::Uniform1i(index, *active_texture_id as i32);
}
// Set a vec2 f32 uniform
pub unsafe fn set_vec2f32(index: i32, vec: &veclib::Vector2<f32>) {
    gl::Uniform2f(index, vec[0], vec[1]);
}
// Set a vec2 f64 uniform
pub unsafe fn set_vec2f64(index: i32, vec: &veclib::Vector2<f64>) {
    gl::Uniform2d(index, vec[0], vec[1]);
}
// Set a vec2 i32 uniform
pub unsafe fn set_vec2i32(index: i32, vec: &veclib::Vector2<i32>) {
    gl::Uniform2i(index, vec[0], vec[1]);
}
// Set a vec3 f32 uniform
pub unsafe fn set_vec3f32(index: i32, vec: &veclib::Vector3<f32>) {
    gl::Uniform3f(index, vec[0], vec[1], vec[2]);
}
// Set a vec3 f64 uniform
pub unsafe fn set_vec3f64(index: i32, vec: &veclib::Vector3<f64>) {
    gl::Uniform3d(index, vec[0], vec[1], vec[2]);
}
// Set a vec3 i32 uniform
pub unsafe fn set_vec3i32(index: i32, vec: &veclib::Vector3<i32>) {
    gl::Uniform3i(index, vec[0], vec[1], vec[2]);
}
// Set a vec4 f32 uniform
pub unsafe fn set_vec4f32(index: i32, vec: &veclib::Vector4<f32>) {
    gl::Uniform4f(index, vec[0], vec[1], vec[2], vec[3]);
}
// Set a vec4 f64 uniform
pub unsafe fn set_vec4f64(index: i32, vec: &veclib::Vector4<f64>) {
    gl::Uniform4d(index, vec[0], vec[1], vec[2], vec[3]);
}
// Set a vec4 i32 uniform
pub unsafe fn set_vec4i32(index: i32, vec: &veclib::Vector4<i32>) {
    gl::Uniform4i(index, vec[0], vec[1], vec[2], vec[3]);
}
// Set a singular boolean
pub unsafe fn set_bool(index: i32, val: &bool) {
    gl::Uniform1i(index, *val as i32);
}
// Set a vec2 boolean
pub unsafe fn set_vec2bool(index: i32, val: &veclib::Vector2<bool>) {
    gl::Uniform2i(index, val[0] as i32, val[1] as i32);
}
// Set a vec3 boolean
pub unsafe fn set_vec3bool(index: i32, val: &veclib::Vector3<bool>) {
    gl::Uniform3i(index, val[0] as i32, val[1] as i32, val[2] as i32);
}
// Set a vec4 boolean
pub unsafe fn set_vec4bool(index: i32, val: &veclib::Vector4<bool>) {
    gl::Uniform4i(index, val[0] as i32, val[1] as i32, val[2] as i32, val[3] as i32);
}
// Set an atomic counter
pub unsafe fn set_atomic(index: i32, val: &AtomicGroup, binding: &u32) {
    gl::BindBuffer(gl::ATOMIC_COUNTER_BUFFER, val.oid);
    gl::BindBufferBase(gl::ATOMIC_COUNTER_BUFFER, *binding as u32, val.oid);
    gl::BindBuffer(gl::ATOMIC_COUNTER_BUFFER, 0);
    //gl::BindBufferRange(gl::ATOMIC_COUNTER_BUFFER, index as u32, oid, 0, size_of::<u32>() as isize);
}
// Set a shader storage
pub unsafe fn set_shader_storage(index: i32, val: &ShaderStorage, binding: &u32) {
    gl::BindBuffer(gl::SHADER_STORAGE_BUFFER, val.oid);
    gl::BindBufferBase(gl::SHADER_STORAGE_BUFFER, *binding as u32, val.oid);
    gl::BindBuffer(gl::SHADER_STORAGE_BUFFER, 0);
}