use gl::types::GLuint;

use super::{get_texel_byte_size, BundledTexture2D, RawTexture, Texture2D, TextureBytes, TextureParams};
use crate::{basics::texture::TextureFlags, object::PipelineElement, pipeline::Handle};

// Shared texture logic
pub trait Texture {
    // Dimension types
    type Dimensions: Default;

    // Get the raw underlying texture
    fn storage(&self) -> Option<&RawTexture>;
    // Get the texture target (OpenGL)
    fn target(&self) -> Option<GLuint> {
        self.storage().as_ref().map(|storage| storage.target)
    }
    // Get the underlying texture storage name
    fn name(&self) -> Option<GLuint> {
        self.storage().as_ref().map(|storage| storage.name)
    }
    // Get the texture parameters
    fn params(&self) -> &TextureParams;
    // Get the texture bytes
    fn bytes(&self) -> &TextureBytes;
    // Calculate the number of texels in the texture
    fn count_texels(&self) -> usize;
    // Calculate the number of bytes the texture *can* have
    fn count_bytes(&self) -> usize {
        self.count_texels() * get_texel_byte_size(self.params().layout.internal_format)
    }
    // Get the current texture dimensions
    fn dimensions(&self) -> Self::Dimensions;
}

// Resizable texture
pub trait ResizableTexture: Texture {
    // Resize the current texture
    fn resize(&mut self, dimensions: Self::Dimensions) -> Option<()> {
        self.resize_then_write(dimensions, Vec::new())?;
        Some(())
    }
    // Resize the current texture, then set it's bytes
    fn resize_then_write(&mut self, dimensions: Self::Dimensions, bytes: Vec<u8>) -> Option<()>;
}

// Writable texture
pub trait WritableTexture: Texture {
    // Set the contents of the texture
    fn write(&mut self, bytes: Vec<u8>) -> Option<()>;
    // Clear a texture
    fn clear(&mut self) {
        self.write(Vec::new()).unwrap()
    }
}

/*

fn write(&mut self, bytes: Vec<u8>) -> Option<()> {
    // Write to the OpenGL texture first
    let ptr = verify_byte_size(self.count_bytes(), &bytes)?;

    // Write
    if let Some(raw) = self.raw.as_ref() {
        let (width, height, layers) = (self.dimensions).as_::<i32>().into_tuple();
        let ifd = raw.ifd;
        unsafe {
            gl::TexSubImage3D(gl::TEXTURE_2D_ARRAY, 0, 0, 0, 0, width, height, layers, ifd.1, ifd.2, ptr);
        }
    }
    // Then save the bytes if possible
    if self.params.flags.contains(TextureFlags::PERSISTENT) {
        self.bytes = TextureBytes::Valid(bytes);
    }
    Some(())
} */

/*
// Write to the OpenGL texture first
// Manually drop the vector when we are done with it
let manual = ManuallyDrop::new(bytes);
if let Some(raw) = self.raw.as_ref() {
    let (width, height) = (self.dimensions).as_::<i32>().into_tuple();
    unsafe {
        let ptr = verify_byte_size(self.count_bytes(), manual.as_ref())?;
        let ifd = raw.ifd;
        gl::BindTexture(gl::TEXTURE_2D, raw.name);
        gl::TexSubImage2D(gl::TEXTURE_2D, 0, 0, 0, width, height, ifd.1, ifd.2, ptr);
    }
}

// Then save the bytes if possible
if self.params.flags.contains(TextureFlags::PERSISTENT) {
    self.bytes = TextureBytes::Valid(ManuallyDrop::into_inner(manual));
}
Some(())
*/

/*

impl Texture {
    // Count the numbers of pixels that this texture can contain
    pub fn count_texels(&self) -> usize {
        match self.dimensions {
            TextureDimensions::Texture1d(x) => (x as usize),
            TextureDimensions::Texture2d(xy) => (xy.x as usize * xy.y as usize),
            TextureDimensions::Texture3d(xyz) => (xyz.x as usize * xyz.y as usize * xyz.z as usize),
            TextureDimensions::Texture2dArray(xyz) => (xyz.x as usize * xyz.y as usize * xyz.z as usize),
        }
    }
    // Set some new dimensions for this texture
    // This also clears the texture
    pub fn set_dimensions(&mut self, dims: TextureDimensions) -> Result<(), OpenGLObjectNotInitialized> {
        if !self.layout.resizable {
            panic!()
        }
        if self.buffer == 0 {
            return Err(OpenGLObjectNotInitialized);
        }
        // Check if the current dimension type matches up with the new one
        self.dimensions = dims;
        let _ifd = self.ifd;
        // This is a normal texture getting resized
        unsafe {
            gl::BindTexture(self.target, self.buffer);
            init_contents(self.target, self.layout.resizable, self.ifd, self.layout, null(), self.dimensions);
        }
        Ok(())
    }
    // Set the texture's bytes (it's contents)
    pub fn set_bytes(&mut self, bytes: Vec<u8>) -> Result<(), OpenGLObjectNotInitialized> {
        self.bytes = bytes;
        let pointer: *const c_void = self.bytes.as_ptr() as *const c_void;
        unsafe {
            gl::BindTexture(self.target, self.buffer);
            update_contents(self.target, self.ifd, pointer, self.dimensions)
        }
        Ok(())
    }
}

// Initialize the contents of an OpenGL texture using it's dimensions and bytes
unsafe fn init_contents(target: GLuint, resizable: bool, ifd: (GLint, GLuint, GLuint), layout: TextureLayout, pointer: *const c_void, dimensions: TextureDimensions) {
    // Guess how many mipmap levels a texture with a specific maximum coordinate can have
    fn guess_mipmap_levels(i: u16) -> i32 {
        let mut x: f32 = i as f32;
        let mut num: i32 = 0;
        while x > 1.0 {
            // Repeatedly divide by 2
            x /= 2.0;
            num += 1;
        }
        num
    }
    // Get the byte size per texel
    let bsize = convert_format_to_texel_byte_size(layout.internal_format) as isize;

    // Depends if it is resizable or not
    if !resizable {
        // Static
        match dimensions {
            TextureDimensions::Texture1d(width) => {
                gl::TexStorage1D(target, guess_mipmap_levels(width), ifd.0 as u32, width as i32);
                if !pointer.is_null() {
                    // Set a sub-image
                    gl::TexSubImage1D(target, 0, 0, width as i32, ifd.1, ifd.2, pointer);
                }
            }
            // This is a 2D texture
            TextureDimensions::Texture2d(dims) => {
                gl::TexStorage2D(target, guess_mipmap_levels((dims.x).max(dims.y)), ifd.0 as u32, dims.x as i32, dims.y as i32);
                if !pointer.is_null() {
                    // Set a sub-image
                    gl::TexSubImage2D(target, 0, 0, 0, dims.x as i32, dims.y as i32, ifd.1, ifd.2, pointer);
                }
            }
            // This is a 3D texture
            TextureDimensions::Texture3d(dims) => {
                gl::TexStorage3D(
                    target,
                    guess_mipmap_levels((dims.x).max(dims.y).max(dims.z)),
                    ifd.0 as u32,
                    dims.x as i32,
                    dims.y as i32,
                    dims.z as i32,
                );
                if !pointer.is_null() {
                    // Set each sub-image
                    for i in 0..dims.z {
                        let localized_bytes = pointer.offset(i as isize * dims.y as isize * bsize * dims.x as isize) as *const c_void;
                        gl::TexSubImage3D(target, 0, 0, 0, i as i32, dims.x as i32, dims.y as i32, dims.z as i32, ifd.1, ifd.2, localized_bytes);
                    }
                }
            }
            // This is a texture array
            TextureDimensions::Texture2dArray(dims) => {
                gl::TexStorage3D(target, guess_mipmap_levels((dims.x).max(dims.y)), ifd.0 as u32, dims.x as i32, dims.y as i32, dims.z as i32);
                // Set each sub-image
                for i in 0..dims.z {
                    let localized_bytes = pointer.offset(i as isize * dims.y as isize * 4 * dims.x as isize) as *const c_void;
                    gl::TexSubImage3D(target, 0, 0, 0, i as i32, dims.x as i32, dims.y as i32, dims.z as i32, ifd.1, ifd.2, localized_bytes);
                }
            }
        }
    } else {
        // Resizable
        match dimensions {
            TextureDimensions::Texture1d(width) => {
                gl::TexImage1D(target, 0, ifd.0, width as i32, 0, ifd.1, ifd.2, pointer);
            }
            // This is a 2D texture
            TextureDimensions::Texture2d(dims) => {
                gl::TexImage2D(target, 0, ifd.0, dims.x as i32, dims.y as i32, 0, ifd.1, ifd.2, pointer);
            }
            // This is a 3D texture
            TextureDimensions::Texture3d(dims) => {
                gl::TexImage3D(target, 0, ifd.0, dims.x as i32, dims.y as i32, dims.z as i32, 0, ifd.1, ifd.2, pointer);
            }
            // This is a texture array
            TextureDimensions::Texture2dArray(dims) => {
                gl::TexImage3D(target, 0, ifd.0, dims.x as i32, dims.y as i32, dims.z as i32, 0, ifd.1, ifd.2, pointer);
            }
        }
    }
}

// Update the contents of an already existing OpenGL texture
unsafe fn update_contents(target: GLuint, , pointer: *const c_void, dimensions: TextureDimensions) {
    match dimensions {
        TextureDimensions::Texture1d(width) => {
            gl::TexSubImage1D(target, 0, 0, width as i32, ifd.1, ifd.2, pointer);
        }
        // This is a 2D texture
        TextureDimensions::Texture2d(dims) => {
            gl::TexSubImage2D(target, 0, 0, 0, dims.x as i32, dims.y as i32, ifd.1, ifd.2, pointer);
        }
        // This is a 3D texture
        TextureDimensions::Texture3d(dims) => {
            gl::TexSubImage3D(target, 0, 0, 0, 0, dims.x as i32, dims.y as i32, dims.z as i32, ifd.1, ifd.2, pointer);
        }
        // This is a texture array
        TextureDimensions::Texture2dArray(dims) => {
            gl::TexSubImage3D(target, 0, 0, 0, 0, dims.x as i32, dims.y as i32, dims.z as i32, ifd.1, ifd.2, pointer);
        }
    }
}
*/
/*
// Get OpenGL internal format, format, and data type
self.ifd = get_ifd(self.layout);
self.target = match self.dimensions {
    TextureDimensions::Texture1d(_) => gl::TEXTURE_1D,
    TextureDimensions::Texture2d(_) => gl::TEXTURE_2D,
    TextureDimensions::Texture3d(_) => gl::TEXTURE_3D,
    TextureDimensions::Texture2dArray(_) => gl::TEXTURE_2D_ARRAY,
};
// Get the pointer to the bytes data
let pointer: *const c_void = if !self.bytes.is_empty() { self.bytes.as_ptr() as *const c_void } else { null() };

// Convert the textures to SRGBA textures if needed
if self.bits.contains(TextureBits::SRGB) && self.ifd.0 == gl::RGBA8 as i32 {
    self.ifd.0 = gl::SRGB8_ALPHA8 as i32;
}

// Create the texture and bind it
unsafe {
    gl::GenTextures(1, &mut self.buffer);
    gl::BindTexture(self.target, self.buffer);
    // Set the texture contents
    init_contents(self.target, self.layout.resizable, self.ifd, self.layout, pointer, self.dimensions);
}

// The texture is already bound
if self.bits.contains(TextureBits::MIPMAPS) {
    unsafe {
        // Create the mipmaps
        gl::GenerateMipmap(self.target);
    }
}



// Custom shadow texture
if self.bits.contains(TextureBits::SHADOWTEX) {
    unsafe {
        gl::TexParameteri(self.target, gl::TEXTURE_COMPARE_MODE, gl::COMPARE_REF_TO_TEXTURE as i32);
        gl::TexParameteri(self.target, gl::TEXTURE_COMPARE_FUNC, gl::GREATER as i32);
    }
}
*/
