use std::marker::PhantomData;

use crate::{context::Context, texture::TextureMode};

use super::Texture;

// A resident texture handle that will be used for bindless textures
pub struct Resident<T: Texture> {
    // Very fun
    texture: T,

    // The raw OpenGL bindless handle
    handle: u64,

    // Unsend + unsync
    _phantom: PhantomData<*const T>,
}

impl<T: Texture> Resident<T> {
    // Create a new resident bindless texture using a texture
    // PS: THIS SHIT IS VERY UNSAFE. This might bring the whole OS down if I fuck up
    // PS 2: The texture must be 
    pub fn new(_ctx: &mut Context, texture: T) -> Self {
        // We can only use make_resident for static & dynamic textures
        assert!(texture.mode() == TextureMode::Static || texture.mode() == TextureMode::Dynamic, "Texture must be initialize with TextureMode::Static or TextureMode::Dynamic");

        // Create the raw handle and make it resident
        let handle = unsafe {
            let handle = gl::GetTextureHandleARB(texture.name().get());
            gl::MakeTextureHandleResidentARB(handle);
            handle
        };

        Self { 
            texture,
            handle,
            _phantom: Default::default()
        }
    }
}

impl<T: Texture> AsMut<T> for Resident<T> {
    fn as_mut(&mut self) -> &mut T {
        todo!()
    }
}