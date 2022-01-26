use crate::{
    object::{ObjectBuildingTask, ObjectID, PipelineObject, PipelineTask},
    pipeline::Pipeline,
    utils::*,
};
use assets::*;
use gl;
use image::{EncodableLayout, GenericImageView};
use crate::basics::Buildable;

use super::{TextureFormat, TextureFilter, TextureWrapping, TextureType, TextureAccessType, get_ifd};

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
    pub(crate) write_pbo: Option<u32>, pub(crate) read_pbo: Option<u32>,
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
            cpu_access: TextureAccessType::all(),
            write_pbo: None, read_pbo: None,
            mipmaps: false,
        }
    }
}

impl PipelineObject for Texture {}

impl Buildable for Texture {
    fn construct_task(mut self, pipeline: &Pipeline) -> (PipelineTask, ObjectID<Self>) {
        // Before we send off the texture to the render thread, we want to make sure that our internal values are updated
        self.ifd = get_ifd(self._format, self._type);
        self.target = match self.ttype {
            TextureType::Texture1D(_) => gl::TEXTURE_1D,
            TextureType::Texture2D(_, _) => gl::TEXTURE_2D,
            TextureType::Texture3D(_, _, _) => gl::TEXTURE_3D,
            TextureType::Texture2DArray(_, _, _) => gl::TEXTURE_2D_ARRAY,
        };
        // Create the ID
        let id = pipeline.textures.get_next_id_increment();
        let id = ObjectID::new(id);
        // Create a task and send it
        (PipelineTask::CreateTexture(ObjectBuildingTask::<Self>(self, id)), id)
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
    // Create a texture array from multiple texture paths (They must have the same dimensions!)
    pub fn create_texturearray(texture_paths: Vec<&str>, width: u16, height: u16) -> Texture {
        // Load the textures
        let mut bytes: Vec<Vec<u8>> = Vec::new();
        for x in &texture_paths {
            // Load this texture from the bytes
            let assetcacher = assets::assetc::asset_cacher();
            let metadata = assetcacher.cached_metadata.get(*x).unwrap();
            let png_bytes = metadata.bytes.as_bytes();
            let image = image::load_from_memory_with_format(png_bytes, image::ImageFormat::Png).unwrap();
            // Resize the image so it fits the dimension criteria
            let image = image.resize_exact(width as u32, height as u32, image::imageops::FilterType::Gaussian);
            // Flip
            let image = image.flipv();
            let bytesa = image.to_bytes();
            bytes.push(bytesa);
        }
        Texture {
            bytes: bytes.into_iter().flatten().collect::<Vec<u8>>(),
            ttype: TextureType::Texture2DArray(width, height, texture_paths.len() as u16),
            ..Texture::default()
        }
    }
}
