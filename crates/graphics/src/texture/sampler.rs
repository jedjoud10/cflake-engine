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
    pub comparison: SamplerComparison
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

// This sampler will be passed to shader groups to allow us
// to read from textures on the GPU
pub struct Sampler<'a, T: Texel> {
    pub(crate) sampler: &'a wgpu::Sampler,
    pub(crate) _phantom: PhantomData<&'a T>,
    pub(crate) settings: &'a SamplerSettings,
}

impl<'a, T: Texel> Sampler<'a, T> {
    // Get internally used raw sampler
    pub fn raw(&self) -> &'a wgpu::Sampler {
        &self.sampler
    }

    // Get the minification filter
    pub fn min_filter(&self) -> SamplerFilter {
        self.settings.min_filter
    }

    // Get the magnification filter
    pub fn mag_filter(&self) -> SamplerFilter {
        self.settings.mag_filter
    }
    
    // Get the mipmapping filter
    pub fn mip_filter(&self) -> SamplerFilter {
        self.settings.mip_filter
    }

    // Get the wrap mode used for the X direction
    pub fn wrap_u(&self) -> SamplerWrap {
        self.settings.wrap_u
    }
    
    // Get the wrap mode used for the Y direction
    pub fn wrap_v(&self) -> SamplerWrap {
        self.settings.wrap_v
    }
    
    // Get the wrap mode used for the z direction
    pub fn wrap_w(&self) -> SamplerWrap {
        self.settings.wrap_w
    }
    
    // Get the sampler border color (if fetching from border color is used within the wraps)
    pub fn border_color(&self) -> wgpu::SamplerBorderColor {
        self.settings.border
    }

    // Get the mipmapping settings used by this sampler
    pub fn mipmap(&self) -> &SamplerMipMaps {
        &self.settings.mipmaps
    }
}

// Convert the mip mapping settings to the anisotropic values used by the Wgpu sampler
pub fn convert_mip_map_anisotropic_clamp(mip_mapping: &SamplerMipMaps) -> u16 {
    match mip_mapping {
        SamplerMipMaps::AutoAniso => 4,
        SamplerMipMaps::ClampedAniso { aniso_samples, .. } | SamplerMipMaps::Aniso { aniso_samples } => aniso_samples.get() as u16,
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

// Tries to fetch an already existing sampler from the graphics context
// If no sampler exist, this will create a completely new one
pub fn get_or_insert_sampler(graphics: &Graphics, sampling: SamplerSettings) -> Arc<wgpu::Sampler> {
    match graphics.0.cached.samplers.entry(sampling) {
        Entry::Occupied(occupied) => {
            occupied.get().clone()
        }
        Entry::Vacant(vacant) => {            
            let anisotropy_clamp = convert_mip_map_anisotropic_clamp(&sampling.mipmaps);
            let (lod_min_clamp, lod_max_clamp) = convert_mip_map_lod_clamp(&sampling.mipmaps);
            let compare = sampling.comparison;
            let border_color = Some(sampling.border);

            // Sampler configuration
            let descriptor = SamplerDescriptor {
                address_mode_u: sampling.wrap_u,
                address_mode_v: sampling.wrap_v,
                address_mode_w: sampling.wrap_w,
                mag_filter: sampling.mag_filter,
                min_filter: sampling.min_filter,
                mipmap_filter: sampling.mip_filter,
                anisotropy_clamp,
                border_color,
                lod_min_clamp,
                lod_max_clamp,
                compare,
                ..Default::default()
            };

            // Create a new sampler and cache it
            let sampler = graphics.device().create_sampler(&descriptor);
            let sampler = Arc::new(sampler);
            vacant.insert(sampler.clone());
            sampler
        }
    }
}
