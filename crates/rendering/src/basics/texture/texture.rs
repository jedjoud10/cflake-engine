use std::{
    ffi::c_void,
    ptr::{null, null_mut},
};

use crate::{
    basics::{texture::calculate_size_bytes, buffer_operation::BufferOperation},
    object::{Construct, ConstructionTask, Deconstruct, DeconstructionTask, GlTracker, ObjectID, OpenGLObjectNotInitialized, PipelineObject},
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

use super::{get_ifd, TextureAccessType, TextureFilter, TextureFormat, TextureType, TextureWrapping};

// A texture
#[derive(Debug)]
pub struct Texture {
    // The OpenGL id for this texture
    pub(crate) oid: GLuint,
    // The bytes stored in this texture
    pub(crate) bytes: Vec<u8>,

    // The internal format of the texture
    pub _format: TextureFormat,
    // The data type that this texture uses for storage
    pub _type: DataType,
    // Internal Format, Format, Data
    pub(crate) ifd: (GLint, GLuint, GLuint),
    // The OpenGL target that is linked with this texture, like TEXTURE_2D or TEXTURE_ARRAY
    pub(crate) target: GLuint,

    // Texture mag and min filters, either Nearest or Linear
    pub filter: TextureFilter,
    // What kind of wrapping will we use for this texture
    pub wrap_mode: TextureWrapping,
    // The border colors
    pub border_colors: [veclib::Vector4<f32>; 4],
    pub custom_params: SmallVec<[(GLuint, GLuint); 2]>,
    // The dimensions of the texture and it's texture type
    pub ttype: TextureType,
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
            ttype: TextureType::Texture2D(0, 0),
            cpu_access: TextureAccessType::empty(),
            dynamic_state: UpdateFrequency::Static,
            write_pbo: None,
            read_pbo: None,
            mipmaps: false,
        }
    }
}
impl PipelineObject for Texture {
    // Reserve an ID for this texture
    fn reserve(self, pipeline: &Pipeline) -> Option<(Self, ObjectID<Self>)> {
        Some((self, pipeline.textures.gen_id()))
    }
    // Send this texture to the pipeline for construction
    fn send(self, id: ObjectID<Self>) -> ConstructionTask {
        ConstructionTask::Texture(Construct::<Self>(self, id))
    }
    // Create a deconstruction task
    fn pull(id: ObjectID<Self>) -> DeconstructionTask {
        DeconstructionTask::Texture(Deconstruct::<Self>(id))
    }
    // Add the texture to our ordered vec
    fn add(mut self, pipeline: &mut Pipeline, id: ObjectID<Self>) -> Option<()> {
        // Add the shader
        self.ifd = get_ifd(self._format, self._type);
        self.target = match self.ttype {
            TextureType::Texture1D(_) => gl::TEXTURE_1D,
            TextureType::Texture2D(_, _) => gl::TEXTURE_2D,
            TextureType::Texture3D(_, _, _) => gl::TEXTURE_3D,
            TextureType::Texture2DArray(_, _, _) => gl::TEXTURE_2D_ARRAY,
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
        let tex_type = match self.ttype {
            TextureType::Texture1D(_) => gl::TEXTURE_1D,
            TextureType::Texture2D(_, _) => gl::TEXTURE_2D,
            TextureType::Texture3D(_, _, _) => gl::TEXTURE_3D,
            TextureType::Texture2DArray(_, _, _) => gl::TEXTURE_2D_ARRAY,
        };
        let texel_count = self.count_pixels();

        let mut oid: u32 = 0;
        unsafe {
            gl::GenTextures(1, &mut oid as *mut u32);
            gl::BindTexture(tex_type, oid);
            if texel_count > 0 {
                match self.ttype {
                    TextureType::Texture1D(width) => {
                        gl::TexImage1D(tex_type, 0, ifd.0, width as i32, 0, ifd.1, ifd.2, pointer);
                    }
                    // This is a 2D texture
                    TextureType::Texture2D(width, height) => {
                        gl::TexImage2D(tex_type, 0, ifd.0, width as i32, height as i32, 0, ifd.1, ifd.2, pointer);
                    }
                    // This is a 3D texture
                    TextureType::Texture3D(width, height, depth) => {
                        gl::TexImage3D(tex_type, 0, ifd.0, width as i32, height as i32, depth as i32, 0, ifd.1, ifd.2, pointer);
                    }
                    // This is a texture array
                    TextureType::Texture2DArray(width, height, depth) => {
                        gl::TexStorage3D(
                            tex_type,
                            guess_mipmap_levels(width.max(height) as usize) as i32,
                            ifd.0 as u32,
                            width as i32,
                            height as i32,
                            depth as i32,
                        );
                        // We might want to do mipmap
                        for i in 0..depth {
                            let localized_bytes = self.bytes[(i as usize * height as usize * 4 * width as usize)..self.bytes.len()].as_ptr() as *const c_void;
                            gl::TexSubImage3D(gl::TEXTURE_2D_ARRAY, 0, 0, 0, i as i32, width as i32, height as i32, 1, ifd.1, ifd.2, localized_bytes);
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

        // Set the wrap mode for the texture (Mipmapped or not)
        let wrapping_mode = match self.wrap_mode {
            TextureWrapping::ClampToEdge => gl::CLAMP_TO_EDGE,
            TextureWrapping::ClampToBorder => gl::CLAMP_TO_BORDER,
            TextureWrapping::Repeat => gl::REPEAT,
            TextureWrapping::MirroredRepeat => gl::MIRRORED_REPEAT,
        };
        unsafe {
            // Now set the actual wrapping mode in the opengl texture
            gl::TexParameteri(tex_type, gl::TEXTURE_WRAP_S, wrapping_mode as i32);
            gl::TexParameteri(tex_type, gl::TEXTURE_WRAP_T, wrapping_mode as i32);
            // And also border colors
            use veclib::Vector;
            let ptr = self.border_colors.get(0).unwrap().as_ptr();
            gl::TexParameterfv(tex_type, gl::TEXTURE_BORDER_COLOR, ptr);
        }

        // Set the custom parameter
        for (name, param) in &self.custom_params {
            unsafe {
                gl::TexParameteri(tex_type, *name, *param as i32);
            }
        }

        // Add the texture
        self.oid = oid;
        unsafe {
            gl::BindTexture(tex_type, 0);
        }

        // If we are a static texture, we don't need to keep the bytes on the CPU anymore
        if let UpdateFrequency::Static = self.dynamic_state {
            self.bytes.drain(..);
        }

        pipeline.textures.insert(id, self);
        Some(())
    }
    // Remove the texture from the pipeline
    fn delete(pipeline: &mut Pipeline, id: ObjectID<Self>) -> Option<Self> {
        let texture = pipeline.textures.remove(id)?;
        // Dispose of the OpenGL buffers
        unsafe {
            gl::DeleteTextures(1, &texture.oid);
            if let Some(x) = texture.read_pbo {
                gl::DeleteBuffers(1, &x)
            }
            if let Some(x) = texture.write_pbo {
                gl::DeleteBuffers(1, &x)
            }
        }
        Some(texture)
    }
}

// Create a texture and send it to the pipeline so we can actually create it on the GPU
impl Texture {
    // The internal format and data type of the soon to be generated texture
    pub fn with_format(mut self, _format: TextureFormat) -> Self {
        self._format = _format;
        self
    }
    // Set the data type for this texture
    pub fn with_data_type(mut self, _type: DataType) -> Self {
        self._type = _type;
        self
    }
    // Set the height and width of the soon to be generated texture
    pub fn with_dimensions(mut self, ttype: TextureType) -> Self {
        self.ttype = ttype;
        self
    }
    // Set the texture type
    pub fn with_type(mut self, ttype: TextureType) -> Self {
        self.ttype = ttype;
        self
    }
    // Set the bytes of this texture
    pub fn with_bytes(mut self, bytes: Vec<u8>) -> Self {
        self.bytes = bytes;
        self
    }
    // We can read from this texture on the CPU, so we must create a Download PBO
    pub fn become_readable(mut self) -> Self {
        self.cpu_access.insert(TextureAccessType::READ);
        self
    }
    // We can write to this texture on the CPU, so we must create an Upload PBO
    pub fn become_writable(mut self) -> Self {
        self.cpu_access.insert(TextureAccessType::WRITE);
        self
    }
    // Set mipmaps
    pub fn with_mipmaps(mut self, enabled: bool) -> Self {
        self.mipmaps = enabled;
        self
    }
    // Set the mag and min filters
    pub fn with_filter(mut self, filter: TextureFilter) -> Self {
        self.filter = filter;
        self
    }
    // Set the wrapping mode
    pub fn with_wrapping_mode(mut self, wrapping_mode: TextureWrapping) -> Self {
        self.wrap_mode = wrapping_mode;
        self
    }
    // Set the border colors
    pub fn with_border_colors(mut self, colors: [veclib::Vector4<f32>; 4]) -> Self {
        self.border_colors = colors;
        self
    }
    // Set an OpenGL texture parameter for this texture
    pub fn with_custom_gl_param(mut self, name: u32, param: u32) -> Self {
        self.custom_params.push((name, param));
        self
    }
    // Zip up all the pixel bytes from multiple textures
    pub fn pack_bytes(textures: &[&Texture]) -> Option<Vec<u8>> {
        // Load the bytes
        let mut bytes: Vec<u8> = Vec::new();
        let width = textures.get(0)?.ttype.get_width();
        let height = textures.get(0)?.ttype.get_height();
        for texture in textures {
            // Check if we have the same settings
            if texture.ttype.get_height() != height || texture.ttype.get_width() != width {
                return None;
            }
            bytes.extend(texture.bytes.iter());
        }
        Some(bytes)
    }
    // Convert an array of CPU textures to a TextureArray
    // This will use the settings of the first texture in the array
    pub fn convert_texturearray(textures: Vec<&Texture>) -> Option<Texture> {
        let width = textures.get(0)?.ttype.get_width();
        let height = textures.get(0)?.ttype.get_height();
        Some(Texture {
            bytes: Self::pack_bytes(&textures)?,
            ttype: TextureType::Texture2DArray(width, height, textures.len() as u16),
            ..Texture::default()
        })
    }
    // Convert an array of CPU textures to a 3D Texture
    pub fn convert_3d(textures: Vec<&Texture>) -> Option<Texture> {
        let width = textures.get(0)?.ttype.get_width();
        let height = textures.get(0)?.ttype.get_height();
        Some(Texture {
            bytes: Self::pack_bytes(&textures)?,
            ttype: TextureType::Texture3D(width, height, textures.len() as u16),
            ..Texture::default()
        })
    }
    // Count the numbers of pixels that this texture can contain
    pub fn count_pixels(&self) -> usize {
        match self.ttype {
            TextureType::Texture1D(x) => (x as usize),
            TextureType::Texture2D(x, y) => (x as usize * y as usize),
            TextureType::Texture3D(x, y, z) => (x as usize * y as usize * z as usize),
            TextureType::Texture2DArray(x, y, z) => (x as usize * y as usize * z as usize),
        }
    }
    // Set the inner data of the texture, and resize it
    pub fn update_size_fill(&mut self, tt: TextureType, bytes: Vec<u8>) -> Result<(), OpenGLObjectNotInitialized> {
        if self.oid == 0 {
            return Err(OpenGLObjectNotInitialized);
        }

        let pointer: *const c_void = if !bytes.is_empty() { bytes.as_ptr() as *const c_void } else { null() };

        // Check if the current dimension type matches up with the new one
        self.ttype = tt;
        let ifd = self.ifd;
        // This is a normal texture getting resized
        unsafe {
            match tt {
                TextureType::Texture1D(width) => {
                    gl::BindTexture(gl::TEXTURE_1D, self.oid);
                    gl::TexImage1D(gl::TEXTURE_2D, 0, ifd.0, width as i32, 0, ifd.1, ifd.2, pointer);
                }
                TextureType::Texture2D(width, height) => {
                    gl::BindTexture(gl::TEXTURE_2D, self.oid);
                    gl::TexImage2D(gl::TEXTURE_2D, 0, ifd.0, width as i32, height as i32, 0, ifd.1, ifd.2, pointer);
                }
                TextureType::Texture3D(width, height, depth) => {
                    gl::BindTexture(gl::TEXTURE_3D, self.oid);
                    gl::TexImage3D(gl::TEXTURE_3D, 0, ifd.0, width as i32, height as i32, depth as i32, 0, ifd.1, ifd.2, pointer);
                }
                TextureType::Texture2DArray(_, _, _) => todo!(),
            }
        }
        Ok(())
    }
    // Read/write the bytes
    pub(crate) fn buffer_operation(&self, pipeline: &Pipeline, op: BufferOperation) -> GlTracker {
        let read = if let BufferOperation::Read(rb) = op { rb } else { panic!() };
        // Actually read the pixels
        let read_pbo = self.read_pbo;
        let byte_count = calculate_size_bytes(&self._format, self.count_pixels());
        GlTracker::new(
            |_pipeline| unsafe {
                // Bind the buffer before reading
                gl::BindBuffer(gl::PIXEL_PACK_BUFFER, self.read_pbo.unwrap());
                gl::BindTexture(self.target, self.oid);
                let (_internal_format, format, data_type) = self.ifd;
                gl::GetTexImage(self.target, 0, format, data_type, null_mut());
            },
            pipeline,
        )
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

impl Asset for Texture {
    fn deserialize(self, _meta: &assets::metadata::AssetMetadata, bytes: &[u8]) -> Option<Self>
    where
        Self: Sized,
    {
        // Read bytes
        pub fn read_bytes(bytes: &[u8]) -> (Vec<u8>, u16, u16) {
            // Load this texture from the bytes
            let image = image::load_from_memory(bytes).unwrap();
            let image = image::DynamicImage::ImageRgba8(image.into_rgba8());
            // Flip
            let image = image.flipv();
            (image.to_bytes(), image.width() as u16, image.height() as u16)
        }
        // Load this texture from the bytes
        let (bytes, width, height) = read_bytes(bytes);

        // Return a texture with the default parameters
        let texture = self
            .with_bytes(bytes)
            .with_dimensions(TextureType::Texture2D(width, height))
            .with_format(TextureFormat::RGBA8R)
            .with_data_type(DataType::U8);
        Some(texture)
    }
}
