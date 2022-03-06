use std::{
    ffi::c_void,
    ptr::{null, null_mut},
};

use crate::{
    basics::{buffer_operation::BufferOperation, texture::calculate_size_bytes},
    object::{OpenGLObjectNotInitialized, PipelineCollectionElement},
    pipeline::{Handle, Pipeline},
    utils::*,
};

use super::{get_ifd, TextureAccessType, TextureDimensions, TextureFilter, TextureFormat, TextureWrapping};
use assets::Asset;
use getset::{CopyGetters, Getters};
use gl::{
    self,
    types::{GLint, GLuint},
};
use image::GenericImageView;
use smallvec::SmallVec;

// A texture
#[derive(CopyGetters, Getters)]
pub struct Texture {
    // The OpenGL id for this texture
    #[getset(get_copy = "pub(crate)")]
    oid: GLuint,
    // The bytes stored in this texture
    #[getset(get = "pub")]
    bytes: Vec<u8>,

    // The internal format of the texture
    #[getset(get_copy = "pub")]
    _format: TextureFormat,
    // The data type that this texture uses for storage
    #[getset(get_copy = "pub")]
    _type: DataType,
    // Internal Format, Format, Data
    #[getset(get_copy = "pub(crate)")]
    ifd: (GLint, GLuint, GLuint),
    // The OpenGL target that is linked with this texture, like TEXTURE_2D or TEXTURE_ARRAY
    #[getset(get_copy = "pub(crate)")]
    target: GLuint,

    // Texture mag and min filters, either Nearest or Linear
    #[getset(get_copy = "pub")]
    filter: TextureFilter,
    // What kind of wrapping will we use for this texture
    #[getset(get_copy = "pub")]
    wrap_mode: TextureWrapping,

    // The border colors
    #[getset(get = "pub")]
    custom_params: SmallVec<[(GLuint, GLuint); 2]>,

    // The dimensions of the texture
    #[getset(get_copy = "pub")]
    dimensions: TextureDimensions,

    // TODO: Re-implement the texture download/upload buffers with dynamic/static/stream textures

    // Should we generate mipmaps for this texture
    #[getset(get_copy = "pub")]
    mipmaps: bool,
}

impl Default for Texture {
    fn default() -> Self {
        Self {
            oid: 0,
            bytes: Vec::new(),

            _format: TextureFormat::RGBA8R,
            _type: DataType::U8,
            ifd: get_ifd(TextureFormat::RGBA8R, DataType::U8),
            target: gl::TEXTURE_2D,

            filter: TextureFilter::Linear,
            wrap_mode: TextureWrapping::Repeat,
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
    pub fn _format(mut self, _format: TextureFormat) -> Self {
        self.inner._format = _format;
        self
    }
    pub fn _type(mut self, _type: DataType) -> Self {
        self.inner._type = _type;
        self
    }
    pub fn filter(mut self, filter: TextureFilter) -> Self {
        self.inner.filter = filter;
        self
    }
    pub fn wrap_mode(mut self, wrapping: TextureWrapping) -> Self {
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
        if self.oid == 0 {
            return Err(OpenGLObjectNotInitialized);
        }
        // Check if the current dimension type matches up with the new one
        self.dimensions = dims;
        let ifd = self.ifd;
        // This is a normal texture getting resized
        unsafe {
            gl::BindTexture(self.target, self.oid);
            init_contents(self.target, self.ifd, null(), self.dimensions);
        }
        Ok(())
    }
    // Set the texture's bytes (it's contents)
    pub fn set_bytes(&mut self, bytes: Vec<u8>) -> Result<(), OpenGLObjectNotInitialized> {
        self.bytes = bytes;
        let pointer: *const c_void = self.bytes.as_ptr() as *const c_void;
        unsafe {
            gl::BindTexture(self.target, self.oid);
            update_contents(self.target, self.ifd, pointer, self.dimensions)
        }
        Ok(())
    }
    /*
    // Read/write the bytes
    pub(crate) fn buffer_operation(&self, op: BufferOperation) -> GlTracker {
        match op {
            BufferOperation::Write(_write) => todo!(),
            BufferOperation::Read(read) => {
                // Actually read the pixels
                let read_pbo = self.read_pbo;
                let byte_count = calculate_size_bytes(&self._format, self.count_pixels());
                GlTracker::new(|| unsafe {
                    // Bind the buffer before reading
                    gl::BindBuffer(gl::PIXEL_PACK_BUFFER, self.read_pbo.unwrap());
                    gl::BindTexture(self.target, self.oid);
                    let (_internal_format, format, data_type) = self.ifd;
                    gl::GetTexImage(self.target, 0, format, data_type, null_mut());
                })
                .with_completed_callback(move |_pipeline| unsafe {
                    // Gotta read back the data
                    let mut vec = vec![0_u8; byte_count];
                    gl::BindBuffer(gl::PIXEL_PACK_BUFFER, read_pbo.unwrap());
                    gl::GetBufferSubData(gl::PIXEL_PACK_BUFFER, 0, byte_count as isize, vec.as_mut_ptr() as *mut c_void);
                    let mut cpu_bytes = read.bytes.as_ref().lock();
                    *cpu_bytes = vec;
                })
            }
        }
    }
    */
}

// Initialize the contents of an OpenGL texture using it's dimensions and bytes
unsafe fn init_contents(target: GLuint, ifd: (GLint, GLuint, GLuint), pointer: *const c_void, dimensions: TextureDimensions) {
    // Guess how many mipmap levels a texture with a specific maximum coordinate can have
    fn guess_mipmap_levels(i: usize) -> usize {
        let mut x: f32 = i as f32;
        let mut num: usize = 0;
        while x > 1.0 {
            // Repeatedly divide by 2
            x /= 2.0;
            num += 1;
        }
        num
    }
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
            gl::TexStorage3D(
                target,
                guess_mipmap_levels((dims.x).max(dims.y) as usize) as i32,
                ifd.0 as u32,
                dims.x as i32,
                dims.y as i32,
                dims.z as i32,
            );
            // We might want to do mipmap
            for i in 0..dims.z {
                let localized_bytes = pointer.offset(i as isize * dims.y as isize * 4 * dims.x as isize) as *const c_void;
                gl::TexSubImage3D(gl::TEXTURE_2D_ARRAY, 0, 0, 0, i as i32, dims.x as i32, dims.y as i32, 1, ifd.1, ifd.2, localized_bytes);
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
            todo!()
        }
    }
}

impl PipelineCollectionElement for Texture {
    fn added(&mut self, collection: &mut crate::pipeline::PipelineCollection<Self>, handle: crate::pipeline::Handle<Self>) {
        self.ifd = get_ifd(self._format, self._type);
        self.target = match self.dimensions {
            TextureDimensions::Texture1d(_) => gl::TEXTURE_1D,
            TextureDimensions::Texture2d(_) => gl::TEXTURE_2D,
            TextureDimensions::Texture3d(_) => gl::TEXTURE_3D,
            TextureDimensions::Texture2dArray(_) => gl::TEXTURE_2D_ARRAY,
        };
        let pointer: *const c_void = if !self.bytes.is_empty() { self.bytes.as_ptr() as *const c_void } else { null() };

        let ifd = get_ifd(self._format, self._type);
        // Get the tex_type based on the TextureDimensionType
        let target = match self.dimensions {
            TextureDimensions::Texture1d(_) => gl::TEXTURE_1D,
            TextureDimensions::Texture2d(_) => gl::TEXTURE_2D,
            TextureDimensions::Texture3d(_) => gl::TEXTURE_3D,
            TextureDimensions::Texture2dArray(_) => gl::TEXTURE_2D_ARRAY,
        };
        unsafe {
            gl::GenTextures(1, &mut self.oid);
            gl::BindTexture(target, self.oid);
            // Set the texture contents
            if !self.bytes.is_empty() {
                init_contents(target, ifd, pointer, self.dimensions);
            }
            // Set the texture parameters for a normal texture
            match self.filter {
                TextureFilter::Linear => {
                    // 'Linear' filter
                    gl::TexParameteri(target, gl::TEXTURE_MIN_FILTER, gl::LINEAR as i32);
                    gl::TexParameteri(target, gl::TEXTURE_MAG_FILTER, gl::LINEAR as i32);
                }
                TextureFilter::Nearest => {
                    // 'Nearest' filter
                    gl::TexParameteri(target, gl::TEXTURE_MIN_FILTER, gl::NEAREST as i32);
                    gl::TexParameteri(target, gl::TEXTURE_MAG_FILTER, gl::NEAREST as i32);
                }
            }
        }

        // The texture is already bound
        if self.mipmaps {
            unsafe {
                // Create the mipmaps
                gl::GenerateMipmap(target);
                // Set the texture parameters for a mipmapped texture
                let (min, mag) = match self.filter {
                    TextureFilter::Linear => {
                        (gl::LINEAR_MIPMAP_LINEAR, gl::LINEAR)
                        // 'Linear' filter
                    }
                    TextureFilter::Nearest => {
                        // 'Nearest' filter
                        (gl::NEAREST_MIPMAP_NEAREST, gl::NEAREST)
                    }
                };
                // Set
                gl::TexParameteri(target, gl::TEXTURE_MIN_FILTER, min as i32);
                gl::TexParameteri(target, gl::TEXTURE_MAG_FILTER, mag as i32);
            }
        }

        // Set the wrap mode for the texture (Mipmapped or not)
        let wrapping_mode = match self.wrap_mode {
            TextureWrapping::ClampToEdge(_) => gl::CLAMP_TO_EDGE,
            TextureWrapping::ClampToBorder(_) => gl::CLAMP_TO_BORDER,
            TextureWrapping::Repeat => gl::REPEAT,
            TextureWrapping::MirroredRepeat => gl::MIRRORED_REPEAT,
        };
        unsafe {
            // Now set the actual wrapping mode in the opengl texture
            gl::TexParameteri(target, gl::TEXTURE_WRAP_S, wrapping_mode as i32);
            gl::TexParameteri(target, gl::TEXTURE_WRAP_T, wrapping_mode as i32);
            // And also border colors
            use veclib::Vector;
            match self.wrap_mode {
                TextureWrapping::ClampToBorder(color) | TextureWrapping::ClampToEdge(color) => {
                    if let Some(color) = color {
                        let ptr = color.as_ptr();
                        gl::TexParameterfv(target, gl::TEXTURE_BORDER_COLOR, ptr);
                    }
                }
                _ => {}
            }
        }

        // Set the custom parameter
        for (name, param) in &self.custom_params {
            unsafe {
                gl::TexParameteri(target, *name, *param as i32);
            }
        }
        unsafe {
            gl::BindTexture(target, 0);
        }
    }

    fn disposed(self) {
        // Dispose of the OpenGL buffers
        unsafe {
            gl::DeleteTextures(1, &self.oid);
        }
    }
}

impl Asset for Texture {
    fn deserialize(self, meta: &assets::metadata::AssetMetadata, bytes: &[u8]) -> Option<Self>
    where
        Self: Sized,
    {
        // Load this texture from the bytes
        // Load this texture from the bytes
        let image = image::load_from_memory(bytes).unwrap();
        let image = image::DynamicImage::ImageRgba8(image.into_rgba8());
        // Flip
        let image = image.flipv();
        let (bytes, width, height) = (image.to_bytes(), image.width() as u16, image.height() as u16);
        None
        /*
        // Return a texture with the default parameters
        let builder = Self::default()
            .with_bytes(bytes)
            .with_dimensions(TextureDimensions::Texture2D(width, height))
            .with_format(TextureFormat::RGBA8R)
            .with_data_type(DataType::U8);
        Some(builder)
        */
    }
}
