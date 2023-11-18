use crate::{ColorTexel, Graphics, Texel, Texture};
use dashmap::mapref::entry::Entry;
use std::{
    collections::hash_map::DefaultHasher,
    hash::{Hash, Hasher},
    marker::PhantomData,
    num::NonZeroU8,
    sync::Arc,
};
use utils::Handle;
use wgpu::{AddressMode, CompareFunction, SamplerDescriptor};

// This enum tells the sampler how it should use the mipmaps from the texture
#[derive(Default, Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum SamplerMipMaps {
    // Sampler will fetch it's required data from the texture (aniso disabled)
    Auto,

    // Sampler will fetch it's required data from the texture with anisotropy
    #[default]
    AutoAniso,

    // Clamped sampler mip mapping levels (aniso disabled)
    Clamped {
        min_lod: NonZeroU8,
        max_lod: NonZeroU8,
    },

    // Clamped sampler mip mapping levels with anisotropy samples
    ClampedAniso {
        min_lod: NonZeroU8,
        max_lod: NonZeroU8,
        aniso_samples: NonZeroU8,
    },

    // Sampler with anisotropy samples
    Aniso {
        aniso_samples: NonZeroU8,
    },
}

// Optional compare function for sampling depth textures
pub type SamplerComparison = Option<wgpu::CompareFunction>;
pub type SamplerWrap = wgpu::AddressMode;
pub type SamplerFilter = wgpu::FilterMode;
pub type SamplerBorderColor = wgpu::SamplerBorderColor;

// Some special sampling parameters for textures
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct SamplerSettings {
    pub mag_filter: SamplerFilter,
    pub min_filter: SamplerFilter,
    pub mip_filter: SamplerFilter,
    pub wrap_u: SamplerWrap,
    pub wrap_v: SamplerWrap,
    pub wrap_w: SamplerWrap,
    pub border: SamplerBorderColor,
    pub mipmaps: SamplerMipMaps,
    pub comparison: SamplerComparison,
}

impl Default for SamplerSettings {
    fn default() -> Self {
        Self {
            border: SamplerBorderColor::OpaqueBlack,
            mipmaps: Default::default(),
            comparison: None,
            mag_filter: SamplerFilter::Linear,
            min_filter: SamplerFilter::Linear,
            mip_filter: SamplerFilter::Linear,
            wrap_u: SamplerWrap::Repeat,
            wrap_v: SamplerWrap::Repeat,
            wrap_w: SamplerWrap::Repeat,
        }
    }
}

// Convert the mip mapping settings to the anisotropic values used by the Wgpu sampler
pub fn convert_mip_map_anisotropic_clamp(mip_mapping: &SamplerMipMaps) -> u16 {
    match mip_mapping {
        SamplerMipMaps::AutoAniso => 4,
        SamplerMipMaps::ClampedAniso { aniso_samples, .. }
        | SamplerMipMaps::Aniso { aniso_samples } => aniso_samples.get() as u16,
        _ => 1,
    }
}

// Convert the mip mapping settings to the LOD clamping values
pub fn convert_mip_map_lod_clamp(mip_mapping: &SamplerMipMaps) -> (f32, f32) {
    match mip_mapping {
        SamplerMipMaps::Clamped { min_lod, max_lod }
        | SamplerMipMaps::ClampedAniso {
            min_lod, max_lod, ..
        } => (min_lod.get() as f32, max_lod.get() as f32),
        _ => (0f32, f32::MAX),
    }
}

/// Sampler struct that will wrap a wgpu sampler
pub struct Sampler(wgpu::Sampler, SamplerSettings);

impl Sampler {
    /// Create a new sampler from the graphics context and some sampler settings
    fn new(graphics: &Graphics, settings: SamplerSettings) -> Self {
        let mut anisotropy_clamp = convert_mip_map_anisotropic_clamp(&settings.mipmaps);
        let (lod_min_clamp, lod_max_clamp) = convert_mip_map_lod_clamp(&settings.mipmaps);
        let compare = settings.comparison;
        let border_color = Some(settings.border);

        if !matches!(settings.mag_filter, SamplerFilter::Linear)
            || !matches!(settings.min_filter, SamplerFilter::Linear)
            || !matches!(settings.mip_filter, SamplerFilter::Linear)
        {
            anisotropy_clamp = 1;
        }

        // Sampler configuration
        let descriptor = SamplerDescriptor {
            address_mode_u: settings.wrap_u,
            address_mode_v: settings.wrap_v,
            address_mode_w: settings.wrap_w,
            mag_filter: settings.mag_filter,
            min_filter: settings.min_filter,
            mipmap_filter: settings.mip_filter,
            anisotropy_clamp,
            border_color,
            lod_min_clamp,
            lod_max_clamp,
            compare,
            ..Default::default()
        };

        // Create a new sampler and cache it
        let sampler = graphics.device().create_sampler(&descriptor);
        Self(sampler, settings)
    }

    // Get internally used raw sampler
    pub fn raw(&self) -> &wgpu::Sampler {
        &self.0
    }

    // Get the minification filter
    pub fn min_filter(&self) -> SamplerFilter {
        self.1.min_filter
    }

    // Get the magnification filter
    pub fn mag_filter(&self) -> SamplerFilter {
        self.1.mag_filter
    }

    // Get the mipmapping filter
    pub fn mip_filter(&self) -> SamplerFilter {
        self.1.mip_filter
    }

    // Get the wrap mode used for the X direction
    pub fn wrap_u(&self) -> SamplerWrap {
        self.1.wrap_u
    }

    // Get the wrap mode used for the Y direction
    pub fn wrap_v(&self) -> SamplerWrap {
        self.1.wrap_v
    }

    // Get the wrap mode used for the z direction
    pub fn wrap_w(&self) -> SamplerWrap {
        self.1.wrap_w
    }

    // Get the sampler border color (if fetching from border color is used within the wraps)
    pub fn border_color(&self) -> wgpu::SamplerBorderColor {
        self.1.border
    }

    // Get the mipmapping settings used by this sampler
    pub fn mipmap(&self) -> &SamplerMipMaps {
        &self.1.mipmaps
    }
}