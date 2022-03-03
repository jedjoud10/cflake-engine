use std::{
    ffi::c_void,
    ptr::{null, null_mut},
};

use crate::{
    basics::{buffer_operation::BufferOperation, texture::calculate_size_bytes},
    object::OpenGLObjectNotInitialized,
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
use getset::Getters;
use super::{get_ifd, TextureAccessType, TextureFilter, TextureFormat, TextureDimensions, TextureWrapping};

// A texture
#[derive(Getters)]
pub struct Texture {
    // The OpenGL id for this texture
    pub(crate) oid: GLuint,
    // The bytes stored in this texture
    #[getset(get = "pub")]
    bytes: Vec<u8>,

    // The internal format of the texture
    #[getset(get = "pub")]
    _format: TextureFormat,
    // The data type that this texture uses for storage
    #[getset(get = "pub")]
    _type: DataType,
    // Internal Format, Format, Data
    #[getset(get = "pub")]
    ifd: (GLint, GLuint, GLuint),
    // The OpenGL target that is linked with this texture, like TEXTURE_2D or TEXTURE_ARRAY
    pub(crate) target: GLuint,

    // Texture mag and min filters, either Nearest or Linear
    #[getset(get = "pub")]
    filter: TextureFilter,
    // What kind of wrapping will we use for this texture
    #[getset(get = "pub")]
    wrap_mode: TextureWrapping,
    // The border colors
    #[getset(get = "pub")]
    border_colors: [veclib::Vector4<f32>; 4],
    pub custom_params: SmallVec<[(GLuint, GLuint); 2]>,
    // The dimensions of the texture
    #[getset(get = "pub")]
    dimensions: TextureDimensions,
    // How we access this texture on the CPU
    pub(crate) cpu_access: TextureAccessType,
    // Is this texture dynamic
    pub(crate) dynamic_state: UpdateFrequency,
    // And the corresponding upload / download PBOs,
    pub(crate) write_pbo: Option<GLuint>,
    pub(crate) read_pbo: Option<GLuint>,
    // Should we generate mipmaps for this texture
    pub mipmaps: bool,
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
            border_colors: [veclib::Vector4::<f32>::ZERO; 4],
            custom_params: SmallVec::default(),
            dimensions: TextureDimensions::Texture2D(0, 0),
            cpu_access: TextureAccessType::empty(),
            dynamic_state: UpdateFrequency::Static,
            write_pbo: None,
            read_pbo: None,
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

impl Drop for Texture {
    fn drop(&mut self) {
        // Dispose of the OpenGL buffers
        unsafe {
            gl::DeleteTextures(1, &self.oid);
            if let Some(x) = self.read_pbo {
                gl::DeleteBuffers(1, &x)
            }
            if let Some(x) = self.write_pbo {
                gl::DeleteBuffers(1, &x)
            }
        }
    }
}