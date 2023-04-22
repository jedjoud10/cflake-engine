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
use wgpu::{AddressMode, CompareFunction, SamplerBorderColor, SamplerDescriptor};

// Wrapping mode utilized by the sampler address mode
#[derive(Default, Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum SamplerWrap {
    // Repeats the edge color infinitely
    ClampToEdge,

    // Specific border color
    ClampToBorder(wgpu::SamplerBorderColor),

    // Repeats the texture infinitely
    #[default]
    Repeat,

    // Repeats the texture, but mirrors it to remove seams
    MirroredRepeat,
}

#[derive(Default, Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum SamplerFilter {
    Nearest,

    #[default]
    Linear,
}

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
}

// Some special sampling parameters for textures
#[derive(Default, Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct SamplerSettings {
    pub filter: SamplerFilter,
    pub wrap: SamplerWrap,
    pub mipmaps: SamplerMipMaps,
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

    // Get the filter used by this sampler
    pub fn filter(&self) -> &SamplerFilter {
        &self.settings.filter
    }

    // Get the wrap mode used by this sampler
    pub fn wrap(&self) -> &SamplerWrap {
        &self.settings.wrap
    }

    // Get the mipmapping settings used by this sampler
    pub fn mipmap(&self) -> &SamplerMipMaps {
        &self.settings.mipmaps
    }
}

// Convert the SamplerWrap to AddressMode
pub fn convert_wrap_to_address_mode(
    wrap: &SamplerWrap,
) -> (AddressMode, Option<wgpu::SamplerBorderColor>) {
    match wrap {
        SamplerWrap::ClampToEdge => (wgpu::AddressMode::ClampToEdge, None),
        SamplerWrap::ClampToBorder(color) => (wgpu::AddressMode::ClampToBorder, Some(*color)),
        SamplerWrap::Repeat => (wgpu::AddressMode::Repeat, None),
        SamplerWrap::MirroredRepeat => (wgpu::AddressMode::MirrorRepeat, None),
    }
}

// Convert the sampler filter to the wgpu filter mode
pub fn convert_sampler_filter(filter: SamplerFilter) -> wgpu::FilterMode {
    match filter {
        SamplerFilter::Nearest => wgpu::FilterMode::Nearest,
        SamplerFilter::Linear => wgpu::FilterMode::Linear,
    }
}

// Convert the mip mapping settings to the anisotropic values used by the Wgpu sampler
pub fn convert_mip_map_anisotropic_clamp(mip_mapping: &SamplerMipMaps) -> u16 {
    match mip_mapping {
        SamplerMipMaps::AutoAniso => 16,
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
            log::debug!("Found sampler type in cache, using it...");
            occupied.get().clone()
        }
        Entry::Vacant(vacant) => {
            // Convert texture sampling wrap settings to their Wgpu counterpart
            log::warn!(
                "Did not find cached sampler ({:?}, {:?}, {:?})",
                sampling.filter,
                sampling.wrap,
                sampling.mipmaps
            );
            let (address_mode, border_color) = convert_wrap_to_address_mode(&sampling.wrap);
            let anisotropy_clamp = convert_mip_map_anisotropic_clamp(&sampling.mipmaps);
            let filter = convert_sampler_filter(sampling.filter);
            let (lod_min_clamp, lod_max_clamp) = convert_mip_map_lod_clamp(&sampling.mipmaps);

            // Sampler configuration
            let descriptor = SamplerDescriptor {
                address_mode_u: address_mode,
                address_mode_v: address_mode,
                address_mode_w: address_mode,
                mag_filter: filter,
                min_filter: filter,
                mipmap_filter: filter,
                anisotropy_clamp,
                border_color,
                lod_min_clamp,
                lod_max_clamp,
                ..Default::default()
            };

            // Create a new sampler and cache it
            let sampler = graphics.device().create_sampler(&descriptor);
            let sampler = Arc::new(sampler);
            vacant.insert(sampler.clone());
            log::debug!(
                "Saved sampler ({:?}, {:?}, {:?}) in graphics cache",
                sampling.filter,
                sampling.wrap,
                sampling.mipmaps
            );
            sampler
        }
    }
}
