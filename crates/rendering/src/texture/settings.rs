use std::num::{NonZeroU16, NonZeroU8};

use crate::others::Comparison;

// Some settings that tell us exactly how we should generate a texture
#[derive(Default, Clone, Copy, PartialEq, Eq, Hash, Debug)]
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
            _ => false,
        }
    }
}

// This enum tells the texture how exactly it should create it's mipmaps
// Default mode for mipmap generation is MipMaps::AutomaticAniso
#[derive(Default, Clone, Copy, PartialEq, Eq, Hash)]
pub enum MipMapSetting {
    // Disable mipmap generation for the texture
    Disabled,

    // Automatic mipmap generation based on the texture dimensions
    #[default]
    Automatic,

    // Manual mipmap generation with specific levels.
    // This will be clamped to the maximum number of levels allowed for the given texture dimensions
    // If levels is less than 2, then mipmapping will be disabled
    Manual {
        levels: NonZeroU8,
    },
}

// Texture resolution scale that we can use to downsample or upsample imported textures
pub type TextureResizeFilter = image::imageops::FilterType;
#[derive(Default, Copy, Clone, PartialEq)]
pub enum TextureScale {
    // This will not affect the texture scale
    #[default]
    Default,

    // This will scale the texture size with the "scaling" parameter
    Scale {
        scaling: f64,
        filter: TextureResizeFilter,
    },

    // This will completely resize the texture to a new size
    Resize {
        size: vek::Extent2<NonZeroU16>,
        filter: TextureResizeFilter,
    },
}

// Texel filters that are applied to the texture's mininifcation and magnification parameters
#[derive(Clone, Copy, PartialEq, Eq, Hash)]
pub enum Filter {
    Nearest,
    Linear,
}

// Wrapping mode utilised by TEXTURE_WRAP_R and TEXTURE_WRAP_T
#[derive(Clone, Copy, PartialEq)]
pub enum Wrap {
    ClampToEdge,
    ClampToBorder(vek::Rgba<f32>),
    Repeat,
    MirroredRepeat,
}

// Some special sampling parameters for textures
#[derive(Clone, Copy, PartialEq)]
pub struct Sampling {
    // Minification and magnification filters
    pub filter: Filter,

    // Wrapping mode for each direction
    pub wrap: Wrap,

    // This is a comparison hint used when dealing with depth textures
    pub depth_comparison: Option<Comparison>,

    // Only used when we have texture mipmapping enabled
    pub mipmap_lod_bias: f32,
    pub mipmap_lod_range: (f32, f32),
    pub mipmap_aniso_samples: Option<NonZeroU8>,
}

impl Default for Sampling {
    fn default() -> Self {
        Self {
            filter: Filter::Linear,
            wrap: Wrap::Repeat,
            depth_comparison: None,
            mipmap_lod_bias: 0.0,
            mipmap_lod_range: (-1000.0, 1000.0),
            mipmap_aniso_samples: Some(NonZeroU8::new(4).unwrap()),
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

// What to do when we load in a new cubemap texture
#[derive(Clone, Copy, PartialEq, Eq)]
pub enum CubeMapConvolutionMode {
    // Convolute the cubemap into an environment map that we can use for diffuse lighting
    DiffuseIrradiance,

    // Convolute the cubemap for usage within a specular IBL
    // This requires the cubemap settings to have mipmap enabled
    SpecularIBL,
}