use std::{
    ffi::c_void,
    ptr::{null, null_mut},
};

use crate::{
    basics::{buffer_operation::BufferOperation, texture::calculate_size_bytes},
    object::{OpenGLObjectNotInitialized, PipelineCollectionElement},
    pipeline::Pipeline,
    utils::*,
};

use assets::Asset;
use gl::{
    self,
    types::{GLint, GLuint},
};
use image::GenericImageView;
use smallvec::SmallVec;
use super::{get_ifd, TextureAccessType, TextureFilter, TextureFormat, TextureDimensions, TextureWrapping};

// A texture
pub struct Texture {
    // The OpenGL id for this texture
    glname: GLuint,
    // The bytes stored in this texture
    bytes: Vec<u8>,

    // The internal format of the texture
    _format: TextureFormat,
    // The data type that this texture uses for storage
    _type: DataType,
    // Internal Format, Format, Data
    ifd: (GLint, GLuint, GLuint),
    // The OpenGL target that is linked with this texture, like TEXTURE_2D or TEXTURE_ARRAY
    target: GLuint,

    // Texture mag and min filters, either Nearest or Linear
    filter: TextureFilter,
    // What kind of wrapping will we use for this texture
    wrap_mode: TextureWrapping,

    // The border colors
    custom_params: SmallVec<[(GLuint, GLuint); 2]>,

    // The dimensions of the texture
    dimensions: TextureDimensions,

    // TODO: Re-implement the texture download/upload buffers with dynamic/static/stream textures

    // Should we generate mipmaps for this texture
    mipmaps: bool,
}

// Getters and setters
impl Texture {
    pub(crate) fn glname(&self) -> GLuint { self.glname }
    pub fn bytes(&self) -> &Vec<u8> { self.bytes() }
    pub fn _format(&self) -> TextureFormat { self._format }
    pub fn _type(&self) -> DataType { self._type }
    pub(crate) fn ifd(&self) -> (GLint, GLuint, GLuint) { self.ifd }
    pub(crate) fn target(&self) -> GLuint { self.target }
    pub fn filter(&self) -> TextureFilter { self.filter }
    pub fn wrap_mode(&self) -> TextureWrapping { self.wrap_mode }
    pub fn custom_params(&self) -> &SmallVec<[(GLuint, GLuint); 2]> { &self.custom_params }
    pub fn dimensions(&self) -> &TextureDimensions { &self.dimensions }
    pub fn mipmaps(&self) -> bool { self.mipmaps }
}

impl Default for Texture {
    fn default() -> Self {
        Self {
            glname: 0,
            bytes: Vec::new(),

            _format: TextureFormat::RGBA8R,
            _type: DataType::U8,
            ifd: get_ifd(TextureFormat::RGBA8R, DataType::U8),
            target: gl::TEXTURE_2D,

            filter: TextureFilter::Linear,
            wrap_mode: TextureWrapping::Repeat,
            custom_params: SmallVec::default(),
            dimensions: TextureDimensions::Texture2D(veclib::Vector2::ZERO),
            mipmaps: false,
        }
    }
}
// Create a texture and send it to the pipeline so we can actually create it on the GPU
impl Texture {
    // Count the numbers of pixels that this texture can contain
    pub fn count_pixels(&self) -> usize {
        match self.dimensions {
            TextureDimensions::Texture1D(x) => (x as usize),
            TextureDimensions::Texture2D(xy) => (xy.x as usize * xy.y as usize),
            TextureDimensions::Texture3D(xyz) => (xyz.x as usize * xyz.y as usize * xyz.z as usize),
            TextureDimensions::Texture2DArray(xyz) => (xyz.x as usize * xyz.y as usize * xyz.z as usize),
        }
    }
    /*
    // Zip up all the pixel bytes from multiple textures
    pub fn pack_bytes(textures: &[&Texture]) -> Option<Vec<u8>> {
        // Load the bytes
        let mut bytes: Vec<u8> = Vec::new();
        let width = textures.get(0)?.dimensions.get_width();
        let height = textures.get(0)?.dimensions.get_height();
        for texture in textures {
            // Check if we have the same settings
            if texture.dimensions.get_height() != height || texture.dimensions.get_width() != width {
                return None;
            }
            bytes.extend(texture.bytes.iter());
        }
        Some(bytes)
    }
    // Convert an array of CPU textures to a TextureArray
    // This will use the settings of the first texture in the array
    pub fn convert_texturearray(textures: Vec<&Texture>) -> Option<Texture> {
        let width = textures.get(0)?.dimensions.get_width();
        let height = textures.get(0)?.dimensions.get_height();
        Some(Texture {
            bytes: Self::pack_bytes(&textures)?,
            dimensions: TextureDimensions::Texture2DArray(width, height, textures.len() as u16),
            ..Texture::default()
        })
    }
    // Convert an array of CPU textures to a 3D Texture
    pub fn convert_3d(textures: Vec<&Texture>) -> Option<Texture> {
        let width = textures.get(0)?.dimensions.get_width();
        let height = textures.get(0)?.dimensions.get_height();
        Some(Texture {
            bytes: Self::pack_bytes(&textures)?,
            dimensions: TextureDimensions::Texture3D(width, height, textures.len() as u16),
            ..Texture::default()
        })
    }
    
    // Set the inner data of the texture, and resize it
    pub fn update_size_fill(&mut self, tt: TextureDimensions, bytes: Vec<u8>) -> Result<(), OpenGLObjectNotInitialized> {
        if self.oid == 0 {
            return Err(OpenGLObjectNotInitialized);
        }

        let pointer: *const c_void = if !bytes.is_empty() { bytes.as_ptr() as *const c_void } else { null() };

        // Check if the current dimension type matches up with the new one
        self.dimensions = tt;
        let ifd = self.ifd;
        // This is a normal texture getting resized
        unsafe {
            match tt {
                TextureDimensions::Texture1D(width) => {
                    gl::BindTexture(gl::TEXTURE_1D, self.oid);
                    gl::TexImage1D(gl::TEXTURE_2D, 0, ifd.0, width as i32, 0, ifd.1, ifd.2, pointer);
                }
                TextureDimensions::Texture2D(width, height) => {
                    gl::BindTexture(gl::TEXTURE_2D, self.oid);
                    gl::TexImage2D(gl::TEXTURE_2D, 0, ifd.0, width as i32, height as i32, 0, ifd.1, ifd.2, pointer);
                }
                TextureDimensions::Texture3D(width, height, depth) => {
                    gl::BindTexture(gl::TEXTURE_3D, self.oid);
                    gl::TexImage3D(gl::TEXTURE_3D, 0, ifd.0, width as i32, height as i32, depth as i32, 0, ifd.1, ifd.2, pointer);
                }
                TextureDimensions::Texture2DArray(_, _, _) => todo!(),
            }
        }
        Ok(())
    }
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

impl PipelineCollectionElement for Texture {
    fn added(&mut self, collection: &mut crate::pipeline::PipelineCollection<Self>, handle: crate::pipeline::Handle<Self>) {
        self.ifd = get_ifd(self._format, self._type);
        self.target = match self.dimensions {
            TextureDimensions::Texture1D(_) => gl::TEXTURE_1D,
            TextureDimensions::Texture2D(_) => gl::TEXTURE_2D,
            TextureDimensions::Texture3D(_) => gl::TEXTURE_3D,
            TextureDimensions::Texture2DArray(_) => gl::TEXTURE_2D_ARRAY,
        };
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

        let pointer: *const c_void = if !self.bytes.is_empty() { self.bytes.as_ptr() as *const c_void } else { null() };

        let ifd = get_ifd(self._format, self._type);
        let bytes_count = calculate_size_bytes(&self._format, self.count_pixels());

        // Get the tex_type based on the TextureDimensionType
        let tex_type = match self.dimensions {
            TextureDimensions::Texture1D(_) => gl::TEXTURE_1D,
            TextureDimensions::Texture2D(_) => gl::TEXTURE_2D,
            TextureDimensions::Texture3D(_) => gl::TEXTURE_3D,
            TextureDimensions::Texture2DArray(_) => gl::TEXTURE_2D_ARRAY,
        };
        let texel_count = self.count_pixels();

        let mut oid: u32 = 0;
        unsafe {
            gl::GenTextures(1, &mut oid as *mut u32);
            gl::BindTexture(tex_type, oid);
            if texel_count > 0 {
                match self.dimensions {
                    TextureDimensions::Texture1D(width) => {
                        gl::TexImage1D(tex_type, 0, ifd.0, width as i32, 0, ifd.1, ifd.2, pointer);
                    }
                    // This is a 2D texture
                    TextureDimensions::Texture2D(dims) => {
                        gl::TexImage2D(tex_type, 0, ifd.0, dims.x as i32, dims.y as i32, 0, ifd.1, ifd.2, pointer);
                    }
                    // This is a 3D texture
                    TextureDimensions::Texture3D(dims) => {
                        gl::TexImage3D(tex_type, 0, ifd.0, dims.x as i32, dims.y as i32, dims.z as i32, 0, ifd.1, ifd.2, pointer);
                    }
                    // This is a texture array
                    TextureDimensions::Texture2DArray(dims) => {
                        gl::TexStorage3D(
                            tex_type,
                            guess_mipmap_levels((dims.x).max(dims.y) as usize) as i32,
                            ifd.0 as u32,
                            dims.x as i32,
                            dims.y as i32,
                            dims.z as i32,
                        );
                        // We might want to do mipmap
                        for i in 0..dims.z {
                            let localized_bytes = self.bytes[(i as usize * dims.y as usize * 4 * dims.x as usize)..self.bytes.len()].as_ptr() as *const c_void;
                            gl::TexSubImage3D(gl::TEXTURE_2D_ARRAY, 0, 0, 0, i as i32, dims.x as i32, dims.y as i32, 1, ifd.1, ifd.2, localized_bytes);
                        }
                    }
                }
            }
            // Set the texture parameters for a normal texture
            match self.filter {
                TextureFilter::Linear => {
                    // 'Linear' filter
                    gl::TexParameteri(tex_type, gl::TEXTURE_MIN_FILTER, gl::LINEAR as i32);
                    gl::TexParameteri(tex_type, gl::TEXTURE_MAG_FILTER, gl::LINEAR as i32);
                }
                TextureFilter::Nearest => {
                    // 'Nearest' filter
                    gl::TexParameteri(tex_type, gl::TEXTURE_MIN_FILTER, gl::NEAREST as i32);
                    gl::TexParameteri(tex_type, gl::TEXTURE_MAG_FILTER, gl::NEAREST as i32);
                }
            }
        }

        // The texture is already bound to the TEXTURE_2D
        if self.mipmaps {
            // Create the mipmaps
            unsafe {
                gl::GenerateMipmap(tex_type);
                // Set the texture parameters for a mipmapped texture
                match self.filter {
                    TextureFilter::Linear => {
                        // 'Linear' filter
                        gl::TexParameteri(tex_type, gl::TEXTURE_MIN_FILTER, gl::LINEAR_MIPMAP_LINEAR as i32);
                        gl::TexParameteri(tex_type, gl::TEXTURE_MAG_FILTER, gl::LINEAR as i32);
                    }
                    TextureFilter::Nearest => {
                        // 'Nearest' filter
                        gl::TexParameteri(tex_type, gl::TEXTURE_MIN_FILTER, gl::NEAREST_MIPMAP_NEAREST as i32);
                        gl::TexParameteri(tex_type, gl::TEXTURE_MAG_FILTER, gl::NEAREST as i32);
                    }
                }
            }
        }

        // Create the Upload / Download PBOs if needed
        /*
        if self.cpu_access.contains(TextureAccessType::READ) {
            // Create a download PBO
            let mut pbo = 0_u32;
            unsafe {
                gl::GenBuffers(1, &mut pbo);
                gl::BindBuffer(gl::PIXEL_PACK_BUFFER, pbo);
                gl::BufferData(gl::PIXEL_PACK_BUFFER, bytes_count as isize, null(), gl::STREAM_COPY);
                gl::BindBuffer(gl::PIXEL_PACK_BUFFER, 0);
            }
            self.read_pbo = Some(pbo);
        } else if self.cpu_access.contains(TextureAccessType::WRITE) {
            // Create an upload PBO
            let mut pbo = 0_u32;
            unsafe {
                gl::GenBuffers(1, &mut pbo);
                gl::BindBuffer(gl::PIXEL_UNPACK_BUFFER, pbo);
                gl::BufferData(gl::PIXEL_UNPACK_BUFFER, bytes_count as isize, null(), gl::STREAM_DRAW);
                gl::BindBuffer(gl::PIXEL_UNPACK_BUFFER, 0);
            }
            self.write_pbo = Some(pbo);
        }
        */

        // Set the wrap mode for the texture (Mipmapped or not)
        let wrapping_mode = match self.wrap_mode {
            TextureWrapping::ClampToEdge { border_color: _ } => gl::CLAMP_TO_EDGE,
            TextureWrapping::ClampToBorder { border_color: _ } => gl::CLAMP_TO_BORDER,
            TextureWrapping::Repeat => gl::REPEAT,
            TextureWrapping::MirroredRepeat => gl::MIRRORED_REPEAT,
        };
        unsafe {
            // Now set the actual wrapping mode in the opengl texture
            gl::TexParameteri(tex_type, gl::TEXTURE_WRAP_S, wrapping_mode as i32);
            gl::TexParameteri(tex_type, gl::TEXTURE_WRAP_T, wrapping_mode as i32);
            // And also border colors
            use veclib::Vector;
            match self.wrap_mode {
                TextureWrapping::ClampToBorder { border_color } | TextureWrapping::ClampToEdge { border_color } => {
                    let ptr = border_color.as_ptr();
                    gl::TexParameterfv(tex_type, gl::TEXTURE_BORDER_COLOR, ptr);
                }
                _ => {}
            }            
        }

        // Set the custom parameter
        for (name, param) in &self.custom_params {
            unsafe {
                gl::TexParameteri(tex_type, *name, *param as i32);
            }
        }

        // Add the texture
        self.glname = oid;
        unsafe {
            gl::BindTexture(tex_type, 0);
        }
    }

    fn disposed(self) {
        // Dispose of the OpenGL buffers
        unsafe {
            gl::DeleteTextures(1, &self.glname);
        }
    }
}

impl Asset for Texture {
    fn deserialize(self, meta: &assets::metadata::AssetMetadata, bytes: &[u8]) -> Option<Self>
    where
        Self: Sized {
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