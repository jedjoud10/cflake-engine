use std::{
    ffi::c_void,
    ptr::{null, null_mut},
};

use crate::{
    basics::texture::{calculate_total_texture_byte_size, convert_format_to_texel_byte_size},
    object::{OpenGLObjectNotInitialized, PipelineCollectionElement},
    pipeline::{Handle, Pipeline},
    utils::*,
};

use super::{get_ifd, TextureAccessType, TextureDimensions, TextureFilter, TextureFormat, TextureWrapMode, TextureLayout};
use assets::Asset;
use getset::{CopyGetters, Getters};
use gl::{
    self,
    types::{GLint, GLuint},
};
use image::GenericImageView;
use smallvec::SmallVec;
use veclib::vec2;

// A texture
#[derive(CopyGetters, Getters)]
pub struct Texture {
    // The OpenGL id for this texture
    #[getset(get_copy = "pub(crate)")]
    buffer: GLuint,
    // The bytes stored in this texture
    #[getset(get = "pub")]
    bytes: Vec<u8>,

    // Texture layout
    #[getset(get_copy = "pub")]
    layout: TextureLayout,

    // Internal Format, Format, Data
    #[getset(get_copy = "pub(crate)")]
    ifd: (GLint, GLuint, GLuint),
    // The OpenGL target that is linked with this texture, like TEXTURE_2D or TEXTURE_ARRAY
    #[getset(get_copy = "pub")]
    target: GLuint,

    // Texture mag and min filters, either Nearest or Linear
    #[getset(get_copy = "pub")]
    filter: TextureFilter,
    // What kind of wrapping will we use for this texture
    #[getset(get_copy = "pub")]
    wrap_mode: TextureWrapMode,

    // The border colors
    #[getset(get = "pub")]
    custom_params: SmallVec<[(GLuint, GLuint); 2]>,

    // The dimensions of the texture
    #[getset(get_copy = "pub")]
    dimensions: TextureDimensions,

    // Should we generate mipmaps for this texture
    #[getset(get_copy = "pub")]
    mipmaps: bool,
}

impl Default for Texture {
    fn default() -> Self {
        let layout = TextureLayout {
            data_type: DataType::U8,
            internal_format: TextureFormat::RGBA8R,
            resizable: true,
        };

        Self {
            buffer: 0,
            bytes: Vec::new(),
            layout,
            ifd: get_ifd(layout),
            target: gl::TEXTURE_2D,

            filter: TextureFilter::Linear,
            wrap_mode: TextureWrapMode::Repeat,
            custom_params: SmallVec::default(),
            dimensions: TextureDimensions::Texture2d(veclib::Vector2::ZERO),
            mipmaps: false,
        }
    }
}

// Builder
#[derive(Default)]
pub struct TextureBuilder {
    inner: Texture,
}

impl TextureBuilder {
    // Create a new builder from a texture
    pub fn new(texture: Texture) -> Self {
        Self { inner: texture }
    }
    // This burns my eyes
    pub fn bytes(mut self, bytes: Vec<u8>) -> Self {
        self.inner.bytes = bytes;
        self
    }
    pub fn layout(mut self, layout: TextureLayout) -> Self {
        self.inner.layout = layout;
        self
    }
    pub fn filter(mut self, filter: TextureFilter) -> Self {
        self.inner.filter = filter;
        self
    }
    pub fn wrap_mode(mut self, wrapping: TextureWrapMode) -> Self {
        self.inner.wrap_mode = wrapping;
        self
    }
    pub fn custom_params(mut self, params: &[(GLuint, GLuint)]) -> Self {
        self.inner.custom_params = SmallVec::from_slice(params);
        self
    }
    pub fn dimensions(mut self, dims: TextureDimensions) -> Self {
        self.inner.dimensions = dims;
        self
    }
    pub fn mipmaps(mut self, enabled: bool) -> Self {
        self.inner.mipmaps = enabled;
        self
    }

    // Build
    pub fn build(self) -> Texture {
        self.inner
    }
}

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
        if !self.layout.resizable { panic!() }
        if self.buffer == 0 {
            return Err(OpenGLObjectNotInitialized);
        }
        // Check if the current dimension type matches up with the new one
        self.dimensions = dims;
        let ifd = self.ifd;
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
                gl::TexStorage1D(
                    target, 
                    guess_mipmap_levels(width), 
                    ifd.0 as u32, 
                    width as i32, 
                );
                if !pointer.is_null() {
                    // Set a sub-image
                    gl::TexSubImage1D(target, 0, 0, width as i32, ifd.1, ifd.2, pointer);
                }
            }
            // This is a 2D texture
            TextureDimensions::Texture2d(dims) => {
                gl::TexStorage2D(
                    target, 
                    guess_mipmap_levels((dims.x).max(dims.y)), 
                    ifd.0 as u32, 
                    dims.x as i32, 
                    dims.y as i32,
                );
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
                gl::TexStorage3D(
                    target,
                    guess_mipmap_levels((dims.x).max(dims.y)),
                    ifd.0 as u32,
                    dims.x as i32,
                    dims.y as i32,
                    dims.z as i32,
                );
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
unsafe fn update_contents(target: GLuint, ifd: (GLint, GLuint, GLuint), pointer: *const c_void, dimensions: TextureDimensions) {
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

impl PipelineCollectionElement for Texture {
    fn added(&mut self, handle: &crate::pipeline::Handle<Self>) {
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
        
        // Create the texture and bind it
        unsafe {
            gl::GenTextures(1, &mut self.buffer);
            gl::BindTexture(self.target, self.buffer);
            // Set the texture contents
            init_contents(self.target, self.layout.resizable, self.ifd, self.layout, pointer, self.dimensions);
        }

        // The texture is already bound
        if self.mipmaps {
            unsafe {
                // Create the mipmaps
                gl::GenerateMipmap(self.target);                
            }
        }

        // Texture parameters
        let (min, mag) = if self.mipmaps {
            // Mip-mapped
            match self.filter {
                TextureFilter::Linear => {
                    (gl::LINEAR_MIPMAP_LINEAR, gl::LINEAR)
                    // 'Linear' filter
                }
                TextureFilter::Nearest => {
                    // 'Nearest' filter
                    (gl::NEAREST_MIPMAP_NEAREST, gl::NEAREST)
                }
            }
        } else {
            // Not mip-mapped
            match self.filter {
                TextureFilter::Linear => {
                    (gl::LINEAR, gl::LINEAR)
                    // 'Linear' filter
                }
                TextureFilter::Nearest => {
                    // 'Nearest' filter
                    (gl::NEAREST, gl::NEAREST)
                }
            }
        };
        unsafe {
            // Set
            gl::TexParameteri(self.target, gl::TEXTURE_MIN_FILTER, min as i32);
            gl::TexParameteri(self.target, gl::TEXTURE_MAG_FILTER, mag as i32);
        }

        // Set the wrap mode for the texture (Mipmapped or not)
        let wrap_mode = match self.wrap_mode {
            TextureWrapMode::ClampToEdge(_) => gl::CLAMP_TO_EDGE,
            TextureWrapMode::ClampToBorder(_) => gl::CLAMP_TO_BORDER,
            TextureWrapMode::Repeat => gl::REPEAT,
            TextureWrapMode::MirroredRepeat => gl::MIRRORED_REPEAT,
        };

        unsafe {
            // Now set the actual wrapping mode in the opengl texture
            gl::TexParameteri(self.target, gl::TEXTURE_WRAP_S, wrap_mode as i32);
            gl::TexParameteri(self.target, gl::TEXTURE_WRAP_T, wrap_mode as i32);
            // And also border colors
            use veclib::Vector;
            match self.wrap_mode {
                TextureWrapMode::ClampToBorder(color) | TextureWrapMode::ClampToEdge(color) => {
                    if let Some(color) = color {
                        let ptr = color.as_ptr();
                        gl::TexParameterfv(self.target, gl::TEXTURE_BORDER_COLOR, ptr);
                    }
                }
                _ => {}
            }
        }

        // Set the custom parameters
        for (name, param) in &self.custom_params {
            unsafe {
                gl::TexParameteri(self.target, *name, *param as i32);
            }
        }
    }

    fn disposed(self) {
        // Dispose of the OpenGL buffers
        unsafe {
            gl::DeleteTextures(1, &self.buffer);
        }
    }
}

impl Asset for Texture {
    fn deserialize(self, meta: &assets::metadata::AssetMetadata, bytes: &[u8]) -> Option<Self>
    where
        Self: Sized,
    {
        // Load this texture from the bytes
        let image = image::load_from_memory(bytes).unwrap();
        let image = image::DynamicImage::ImageRgba8(image.into_rgba8());
        // Flip
        let image = image.flipv();
        let (bytes, width, height) = (image.to_bytes(), image.width() as u16, image.height() as u16);
        Some(
            TextureBuilder::default()
                .bytes(bytes)
                .dimensions(TextureDimensions::Texture2d(vec2(width, height)))
                .mipmaps(true)
                .layout(TextureLayout {
                    data_type: DataType::U8,
                    internal_format: TextureFormat::RGBA8R,
                    resizable: false,
                })
                .build(),
        )
    }
}
