use std::{ffi::c_void, mem::ManuallyDrop, ptr::null};

use crate::basics::texture::TextureWrapMode;

use super::{RawTexture, Texture, TextureBytes, TextureFilter, TextureFlags, TextureParams};

// Guess how many mipmap levels a texture can have
// Input value is the maximum dimenions of the texture
pub fn guess_mipmap_levels(i: i32) -> i32 {
    let mut x: f32 = i as f32;
    let mut num: i32 = 0;
    while x > 1.0 {
        // Repeatedly divide by 2
        x /= 2.0;
        num += 1;
    }
    num
}

// Generate mipmaps for a specific texture target
pub unsafe fn generate_mipmaps(target: u32, params: &TextureParams) {
    // Generate mipmaps
    if params.flags.contains(TextureFlags::MIPMAPS) {
        gl::GenerateMipmap(target);
    }

    // Texture filtering
    let (mut min, mag) = match params.filter {
        TextureFilter::Linear => {
            (gl::LINEAR, gl::LINEAR)
            // 'Linear' filter
        }
        TextureFilter::Nearest => {
            // 'Nearest' filter
            (gl::NEAREST, gl::NEAREST)
        }
    };

    // Override if we have mipmapping
    if params.flags.contains(TextureFlags::MIPMAPS) {
        min = match params.filter {
            TextureFilter::Linear => gl::LINEAR_MIPMAP_LINEAR,
            TextureFilter::Nearest => gl::NEAREST_MIPMAP_NEAREST,
        };
    }

    gl::TexParameteri(target, gl::TEXTURE_MIN_FILTER, min as i32);
    gl::TexParameteri(target, gl::TEXTURE_MAG_FILTER, mag as i32);
}

// Generate filters for a specific texture target
pub unsafe fn generate_filters(target: u32, params: &TextureParams) {
    // Set the texture wrap mode
    unsafe fn set_wrap_mode(target: u32, wrap_mode: u32) {
        gl::TexParameteri(target, gl::TEXTURE_WRAP_S, wrap_mode as i32);
        gl::TexParameteri(target, gl::TEXTURE_WRAP_T, wrap_mode as i32);
    }
    // Set the texture's border color
    unsafe fn set_border_color(target: u32, color: Option<vek::Vec4<f32>>) {
        if let Some(color) = color {
            let ptr = color.as_ptr();
            gl::TexParameterfv(target, gl::TEXTURE_BORDER_COLOR, ptr);
        }
    }

    // Set the wrap mode for the texture (Mipmapped or not)
    match params.wrap {
        TextureWrapMode::ClampToEdge(color) => {
            set_wrap_mode(target, gl::CLAMP_TO_EDGE);
            set_border_color(target, color);
        }
        TextureWrapMode::ClampToBorder(color) => {
            set_wrap_mode(target, gl::CLAMP_TO_BORDER);
            set_border_color(target, color);
        }
        TextureWrapMode::Repeat => set_wrap_mode(target, gl::REPEAT),
        TextureWrapMode::MirroredRepeat => set_wrap_mode(target, gl::MIRRORED_REPEAT),
    };
}

// Verify that we can safely write bytes to the texture, then return the pointer to the bytes
pub fn verify_byte_size(byte_size: usize, bytes: &[u8]) -> Option<*const c_void> {
    // Check if the size is legal
    if bytes.len() > byte_size { return None; }
    Some(if bytes.is_empty() {
        null()
    } else {
        bytes.as_ptr() as *const c_void
    })
}

// Store the written bytes into the texture if it's a persistent texture
pub fn store_bytes(flags: TextureFlags, input: Vec<u8>, output: &mut TextureBytes) {
    // If the texture is persistent, save these as our own bytes
    if flags.contains(TextureFlags::PERSISTENT) {
        *output = TextureBytes::Written(input);
    }
}
