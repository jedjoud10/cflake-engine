use std::num::NonZeroU8;

// Some settings that tell us exactly how we should generate a texture
#[derive(Default, Clone, Copy, PartialEq, Eq, Debug)]
pub enum TextureMode {
    // Static textures cannot be modified, they can only be read
    Static,

    // Dynamic textures can be modified throughout their lifetime, but they cannot change size
    Dynamic,

    // Resizable textures are just dynamic textures that can change their size at will
    #[default]
    Resizable,
}

impl TextureMode {
    // Can we read from an arbitrary texture that uses this texture mode?
    pub fn read_permission(&self) -> bool {
        true
    }

    // Can we write to an arbitrary texture that uses this texture mode?
    pub fn write_permission(&self) -> bool {
        match self {
            TextureMode::Static => false,
            _ => true,
        }
    }

    // Can we resize an arbitrary texture that uses this texture mode?
    pub fn resize_permission(&self) -> bool {
        match self {
            TextureMode::Resizable => true,
            _ => false
        }
    }
}

// This enum tells the texture how exactly it should create it's mipmaps
// Default mode for mipmap generation is MipMaps::AutomaticAniso
#[derive(Clone, Copy)]
pub enum MipMapSetting {
    // Disable mipmap generation for the texture
    Disabled,

    // Automatic mipmap generation based on the texture dimensions
    Automatic,

    // Manual mipmap generation with specific levels.
    // This will be clamped to the maximum number of levels allowed for the given texture dimensions
    // If levels is less than 2, then mipmapping will be disabled
    Manual {
        levels: NonZeroU8,
    },

    // Automatic mipmap generation (from texture dimensions), but with a specified number of anisotropy samples
    // The number of anisotropic samples will be decided automatically
    AutomaticAniso,

    // Manual mipmap generation, but with a specified number of anisotropy sampler
    // If levels is less than 2, then mipmapping will be disabled
    // If samples is less than 2m then anisotropic filtering will be disabled
    ManualAniso {
        levels: NonZeroU8,
        samples: NonZeroU8,
    },
}

impl Default for MipMapSetting {
    fn default() -> Self {
        Self::AutomaticAniso
    }
}

// Texel filters that are applied to the texture's mininifcation and magnification parameters
#[derive(Clone, Copy)]
pub enum Filter {
    Nearest,
    Linear,
}

// Wrapping mode utilised by TEXTURE_WRAP_R and TEXTURE_WRAP_T
#[derive(Clone, Copy)]
pub enum Wrap {
    ClampToEdge,
    ClampToBorder(vek::Rgba<f32>),
    Repeat,
    MirroredRepeat,
}

// Some special sampling parameters for textures
#[derive(Clone, Copy)]
pub struct Sampling {
    pub filter: Filter,
    pub wrap: Wrap,
}

impl Default for Sampling {
    fn default() -> Self {
        Self {
            filter: Filter::Linear,
            wrap: Wrap::Repeat,
        }
    }
}

// Texture settings that we shall use when loading in a new texture
#[derive(Default, Clone, Copy)]
pub struct TextureImportSettings {
    pub sampling: Sampling,
    pub mode: TextureMode,
    pub mipmaps: MipMapSetting,
}
