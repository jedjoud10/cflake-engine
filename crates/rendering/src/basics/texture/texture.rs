use crate::{
    object::{ObjectID, PipelineObject, ConstructionTask, Construct},
    pipeline::Pipeline,
    utils::*,
};

use gl;

use super::{get_ifd, TextureAccessType, TextureFilter, TextureFormat, TextureType, TextureWrapping};

// A texture
#[derive(Debug)]
pub struct Texture {
    // The OpenGL id for this texture
    pub(crate) oid: u32,
    // The bytes stored in this texture
    pub(crate) bytes: Vec<u8>,

    // The internal format of the texture
    pub _format: TextureFormat,
    // The data type that this texture uses for storage
    pub _type: DataType,
    // Internal Format, Format, Data
    pub(crate) ifd: (i32, u32, u32),
    // The OpenGL target that is linked with this texture, like TEXTURE_2D or TEXTURE_ARRAY
    pub(crate) target: u32,

    // Texture mag and min filters, either Nearest or Linear
    pub filter: TextureFilter,
    // What kind of wrapping will we use for this texture
    pub wrap_mode: TextureWrapping,
    // The dimensions of the texture and it's texture type
    pub ttype: TextureType,
    // How we access this texture on the CPU
    pub(crate) cpu_access: TextureAccessType,
    // And the corresponding upload / download PBOs,
    pub(crate) write_pbo: Option<u32>,
    pub(crate) read_pbo: Option<u32>,
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
            ttype: TextureType::Texture2D(0, 0),
            cpu_access: TextureAccessType::empty(),
            write_pbo: None,
            read_pbo: None,
            mipmaps: false,
        }
    }
}

impl PipelineObject for Texture {
    // Reserve an ID for this texture
    fn reserve(self, pipeline: &Pipeline) -> Option<(Self, ObjectID<Self>)> where Self: Sized {
        Some((self, ObjectID::new(pipeline.textures.get_next_id_increment())))
    }
    // Send this texture to the pipeline for construction
    fn send(self, pipeline: &Pipeline, id: ObjectID<Self>) -> ConstructionTask {
        ConstructionTask::Texture(Construct::<Self>(self, id))
    }
    // Add the texture to our ordered vec
    fn add(mut self, pipeline: &mut Pipeline, id: ObjectID<Self>) -> Option<()> where Self: Sized {
        // Add the shader
        self.ifd = get_ifd(self._format, self._type);
        self.target = match self.ttype {
            TextureType::Texture1D(_) => gl::TEXTURE_1D,
            TextureType::Texture2D(_, _) => gl::TEXTURE_2D,
            TextureType::Texture3D(_, _, _) => gl::TEXTURE_3D,
            TextureType::Texture2DArray(_, _, _) => gl::TEXTURE_2D_ARRAY,
        };
        pipeline.textures.insert(id.get()?, self);
        Some(())
    }
    // Remove the texture from the pipeline
    fn delete(pipeline: &mut Pipeline, id: ObjectID<Self>) -> Option<Self> where Self: Sized {
        pipeline.textures.remove(id)
    }
}

// Create a texture and send it to the pipeline so we can actually create it on the GPU
impl Texture {
    // The internal format and data type of the soon to be generated texture
    pub fn set_format(mut self, _format: TextureFormat) -> Self {
        self._format = _format;
        self
    }
    // Set the data type for this texture
    pub fn set_data_type(mut self, _type: DataType) -> Self {
        self._type = _type;
        self
    }
    // Set the height and width of the soon to be generated texture
    pub fn set_dimensions(mut self, ttype: TextureType) -> Self {
        self.ttype = ttype;
        self
    }
    // Set the texture type
    pub fn set_type(mut self, ttype: TextureType) -> Self {
        self.ttype = ttype;
        self
    }
    // Set the bytes of this texture
    pub fn set_bytes(mut self, bytes: Vec<u8>) -> Self {
        self.bytes = bytes;
        self
    }
    // We can read from this texture on the CPU, so we must create a Download PBO
    pub fn readable(mut self) -> Self {
        self.cpu_access.insert(TextureAccessType::READ);
        self
    }
    // We can write to this texture on the CPU, so we must create an Upload PBO
    pub fn writable(mut self) -> Self {
        self.cpu_access.insert(TextureAccessType::WRITE);
        self
    }
    // Set mipmaps
    pub fn set_mipmaps(mut self, enabled: bool) -> Self {
        self.mipmaps = enabled;
        self
    }
    // Set the mag and min filters
    pub fn set_filter(mut self, filter: TextureFilter) -> Self {
        self.filter = filter;
        self
    }
    // Set the wrapping mode
    pub fn set_wrapping_mode(mut self, wrapping_mode: TextureWrapping) -> Self {
        self.wrap_mode = wrapping_mode;
        self
    }
    // Zip up all the pixel bytes from multiple textures
    pub fn pack_bytes(textures: &Vec<&Texture>) -> Option<Vec<u8>> {
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
}
