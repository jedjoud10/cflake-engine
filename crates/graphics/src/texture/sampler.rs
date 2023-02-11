use std::{num::NonZeroU8, marker::PhantomData, sync::Arc, collections::hash_map::DefaultHasher, hash::{Hash, Hasher}};
use crate::{Texel, Texture, ColorTexel, Graphics};
use utils::Handle;
use wgpu::{AddressMode, SamplerBorderColor, SamplerDescriptor};
pub use wgpu::FilterMode as SamplerFilter;

// Wrapping mode utilized by the sampler address mode
#[derive(Default, Clone, Copy, PartialEq, Eq, Hash)]
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

// This enum tells the sampler how it should use the mipmaps from the texture
#[derive(Default, Clone, Copy, PartialEq, Eq, Hash)]
pub enum SamplerMipMaps {
    // Sampler will fetch it's required data from the texture (aniso disabled)
    Automatic,

    // Sampler will fetch it's required data from the texture with anisotropy
    #[default]
    AutomaticAniso,

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
    }
}

// Some special sampling parameters for textures
#[derive(Default, Clone, Copy, PartialEq, Eq, Hash)]
pub struct SamplerSettings {
    pub filter: SamplerFilter,
    pub wrap: SamplerWrap,
    pub mipmaps: SamplerMipMaps,
}

// This sampler will be passed to shader groups to allow us
// to read from textures on the GPU
pub struct Sampler<'a, T: Texel> {
    pub(crate) sampler: Arc<wgpu::Sampler>,
    pub(crate) _phantom: PhantomData<&'a T>,
    pub(crate) settings: &'a SamplerSettings,
}

impl<'a, T: Texel> Sampler<'a, T> {
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
pub fn convert_wrap_to_address_mode(wrap: &SamplerWrap) -> (AddressMode, Option<wgpu::SamplerBorderColor>) {
    match wrap {
        SamplerWrap::ClampToEdge => (wgpu::AddressMode::ClampToEdge, None),
        SamplerWrap::ClampToBorder(color) => (wgpu::AddressMode::ClampToBorder, Some(*color)),
        SamplerWrap::Repeat => (wgpu::AddressMode::MirrorRepeat, None),
        SamplerWrap::MirroredRepeat => (wgpu::AddressMode::MirrorRepeat, None),
    }
}

// Convert the mip mapping settings to the anisotropic values used by the Wgpu sampler
pub fn convert_mip_map_anisotropic_clamp(mip_mapping: &SamplerMipMaps) -> Option<NonZeroU8> {
    match mip_mapping {
        SamplerMipMaps::AutomaticAniso => NonZeroU8::new(16),
        _ => None,
    }
}

// Tries to fetch an already existing sampler from the graphics context
// If no sampler exist, this will create a completely new one
pub fn get_or_insert_sampler(
    graphics: &Graphics,
    sampling: SamplerSettings
) -> Arc<wgpu::Sampler> {
    // Hash the sampler settings to get a unique ID
    let hasher = DefaultHasher::new();
    sampling.hash(&mut hasher);
    let hashed = hasher.finish();

    graphics.0.

    if let Some(sampler) = graphics.0.samplers.iter().find(|(_, settings)| *settings == sampling) {
        return &*sampler;
    }
    
    // Convert texture sampling wrap settings to their Wgpu counterpart
    let (address_mode, border_color) = super::convert_wrap_to_address_mode(&sampling.wrap);
    let anisotropy_clamp = super::convert_mip_map_anisotropic_clamp(&sampling.mip_mapping);
    let filter= sampling.filter;

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
        ..Default::default()
    };

    // Create a new sampler and cache it
    let sampler = graphics.device().create_sampler(&descriptor);
    graphics.0.samplers.insert(sampling, sampler);
    &graphics.0.samplers.get(&sampling).unwrap()
    */
}