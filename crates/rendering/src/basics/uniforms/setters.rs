

use crate::{Texture, TextureShaderAccessType};
// Actually set the shader uniforms
#[allow(temporary_cstring_as_ptr)]
// Set a f32 uniform
pub unsafe fn set_f32(index: i32, value: &f32) {
    gl::Uniform1f(index, *value);
}
// Set an image that can be modified inside the shader
pub unsafe fn set_image(texture: &Texture, index: i32, access_type: &TextureShaderAccessType) {
    // Converstion from wrapper to actual opengl values
    let new_access_type: u32;
    match access_type {
        TextureShaderAccessType::ReadOnly => new_access_type = gl::READ_ONLY,
        TextureShaderAccessType::WriteOnly => new_access_type = gl::WRITE_ONLY,
        TextureShaderAccessType::ReadWrite => new_access_type = gl::READ_WRITE,
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
pub unsafe fn set_texture(texture: &Texture, index: i32, active_texture_id: &u32) {
    gl::ActiveTexture(active_texture_id + 33984);
    gl::BindTexture(texture.target, texture.oid);
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
