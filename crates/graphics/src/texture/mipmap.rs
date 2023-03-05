use std::{num::NonZeroU8, ops::DerefMut};

use bytemuck::Zeroable;

use super::{Region, Texture};
use crate::{
    ColorTexel, Extent, MipLevelClearError, MipLevelCopyError,
    MipLevelReadError, MipLevelWriteError, Origin, RenderTarget,
    Texel, TextureAsTargetError, TextureSamplerError, TextureUsage,
};

// This enum tells the texture how exactly it should create it's mipmaps
#[derive(PartialEq, Eq, Hash, Debug)]
pub enum TextureMipMaps<'mip, 'map, T: Texel> {
    // Disable mipmap generation for the texture
    Disabled,

    // Manual mip level generation based on the texture dimensions
    // This will keep the mip levels uninitialized however
    Zeroed {
        // Clamped automatic mipmap generation (to limit number of mips)
        // If levels is less than 2, then mipmapping will be disabled
        // Will be clamped to the maximum number of levels possible
        clamp: Option<NonZeroU8>,
    },

    // Manual mip map generation with the specified texels at each mip level
    // Will be clamped to the maximum number of levels possible

    // NOTE: If this is of length 0 and the texture is loaded from a file, then the
    // mips will be generated automatically based on image file contents
    Manual {
        mips: &'map [&'mip [T::Storage]],
    },
}

impl<T: Texel> Default for TextureMipMaps<'_, '_, T> {
    fn default() -> Self {
        Self::Disabled
    }
}

impl<T: Texel> Copy for TextureMipMaps<'_, '_, T> {}

impl<T: Texel> Clone for TextureMipMaps<'_, '_, T> {
    fn clone(&self) -> Self {
        match self {
            Self::Disabled => Self::Disabled,
            Self::Zeroed { clamp } => Self::Zeroed {
                clamp: clamp.clone(),
            },
            Self::Manual { mips } => {
                Self::Manual { mips: mips.clone() }
            }
        }
    }
}

// Calculate mip levels based on the given color data and size
// Returns None if the texture isn't a power of two texture
pub fn generate_mip_map<T: ColorTexel, E: Extent>(
    base: &[T::Storage],
    extent: E,
) -> Option<Vec<Vec<T::Storage>>> {
    // Convert a xyz value to an index (texel)
    fn xyz_to_index(
        location: vek::Vec3<usize>,
        extent: vek::Extent3<usize>,
    ) -> usize {
        location.x
            + location.y * extent.w
            + location.z * (extent.w * extent.h)
    }

    // Create manual mip maps for this texture
    let dimension = <E as Extent>::dimensionality();
    let name = utils::pretty_type_name::<T>();
    let levels = extent.levels()?.get() as u32;
    let mut map =
        Vec::<Vec<T::Storage>>::with_capacity(levels as usize);
    let mut temp = extent;
    let mut base = base.to_vec();
    log::debug!("Creating mip-data (max = {levels})for imported texture {dimension:?}, <{name}>");

    // Iterate over the levels and fill them up
    // (like how ceddy weddy fills me up inside >.<)
    for i in 0..(levels - 1) {
        // Pre-allocate a vector that will contain the downscaled texels
        let downscaled = extent.mip_level_dimensions(i as u8 + 1);
        let mut texels: Vec<<T as Texel>::Storage> = vec![
            <T::Storage as Zeroable>::zeroed();
            downscaled.area() as usize
        ];

        // Get the original and downscaled sizes
        let original = temp.decompose();
        let new = downscaled.decompose();

        // Division factor is either 2, 4, or 8 (based on dims)
        let factor = match dimension {
            wgpu::TextureDimension::D1 => 2,
            wgpu::TextureDimension::D2 => 4,
            wgpu::TextureDimension::D3 => 8,
        };

        log::debug!(
            "Create mipdata for layer <{i}> from imported image, {}x{}x{}",
            downscaled.width(),
            downscaled.height(),
            downscaled.depth()
        );

        // Write to the downscaled texels
        for ox in 0..original.w {
            for oy in 0..original.h {
                for oz in 0..original.d {
                    // Get the current texel value
                    let texel = base[xyz_to_index(
                        vek::Vec3::new(ox, oy, oz).as_::<usize>(),
                        original.as_::<usize>(),
                    )];

                    // La division est vraiment importante pour qu'on evite un overflow
                    let texel = T::divide(texel, factor as f32);

                    // Get the destination texel value
                    let dst = &mut texels[xyz_to_index(
                        vek::Vec3::new(ox / 2, oy / 2, oz / 2)
                            .as_::<usize>(),
                        new.as_::<usize>(),
                    )];

                    // Sum to the destination
                    *dst += texel;
                }
            }
        }

        // Overwrite temp buffers
        temp = downscaled;
        base[..(downscaled.area() as usize)].copy_from_slice(&texels);
        map.push(texels);
    }

    Some(map)
}

// An immutable mip level that we can use to read from the texture
pub struct MipLevelRef<'a, T: Texture> {
    texture: &'a T,
    level: u8,
}

// Helper methods
impl<'a, T: Texture> MipLevelRef<'a, T> {
    // Get the underlying texture
    pub fn texture(&self) -> &T {
        self.texture
    }

    // Get the mip level of the current level
    pub fn level(&self) -> u8 {
        self.level
    }

    // Get the view for this mip
    pub fn view(&self) -> &wgpu::TextureView {
        &self.texture().views()[self.level as usize]
    }

    // Get the mip level's dimensions
    pub fn dimensions(&self) -> <T::Region as Region>::E {
        self.texture.dimensions().mip_level_dimensions(self.level)
    }

    // Get the mip level's region
    pub fn region(&self) -> T::Region {
        T::Region::with_extent(self.dimensions())
    }
}

impl<'a, T: Texture> MipLevelRef<'a, T> {
    // Read some pixels from the mip level region to the given destination
    pub fn read(
        &self,
        dst: &mut [<T::T as Texel>::Storage],
        subregion: Option<T::Region>,
    ) -> Result<(), MipLevelReadError> {
        // Nothing to write to
        if dst.is_empty() {
            return Ok(());
        }

        // Make sure we can read from the texture
        if !self.texture.usage().contains(TextureUsage::READ) {
            return Err(MipLevelReadError::NonReadable);
        }

        // Get the region for this mip level
        let mip_level_region = <T::Region as Region>::with_extent(
            self.texture
                .dimensions()
                .mip_level_dimensions(self.level),
        );

        // Make sure the "offset" doesn't cause reads outside the texture
        if let Some(subregion) = subregion {
            if mip_level_region.is_larger_than(subregion) {
                return Err(MipLevelReadError::InvalidRegion());
            }
        }

        // Get the mip level subregion if the given one is None
        let subregion = subregion.unwrap_or(mip_level_region);

        // TODO: Actually handle reading here
        todo!();
    }
}

// A mutable mip level that we can use to write to the texture
pub struct MipLevelMut<'a, T: Texture> {
    texture: &'a T,
    level: u8,
}

// Helper methods
impl<'a, T: Texture> MipLevelMut<'a, T> {
    // Get the underlying texture
    pub fn texture(&self) -> &T {
        self.texture
    }

    // Get the mip level of the current level
    pub fn level(&self) -> u8 {
        self.level
    }

    // Get the view for this mip
    pub fn view(&self) -> &wgpu::TextureView {
        &self.texture().views()[self.level as usize]
    }

    // Get the mip level's dimensions
    pub fn dimensions(&self) -> <T::Region as Region>::E {
        self.texture.dimensions().mip_level_dimensions(self.level)
    }

    // Get the mip level's region
    pub fn region(&self) -> T::Region {
        T::Region::with_extent(self.dimensions())
    }

    // Try to get a render target so we can render to this one mip level
    pub fn as_render_target(
        &mut self,
    ) -> Result<RenderTarget<T::T>, TextureAsTargetError> {
        todo!()
    }
}

impl<'a, T: Texture> MipLevelMut<'a, T> {
    // Read some pixels from the mip level region to the given destination
    pub fn read(
        &self,
        dst: &mut [<T::T as Texel>::Storage],
        subregion: Option<T::Region>,
    ) -> Result<(), MipLevelReadError> {
        // Nothing to write to
        if dst.is_empty() {
            return Ok(());
        }

        // Make sure we can read from the texture
        if !self.texture.usage().contains(TextureUsage::READ) {
            return Err(MipLevelReadError::NonReadable);
        }

        // Get the region for this mip level
        let mip_level_region = <T::Region as Region>::with_extent(
            self.texture
                .dimensions()
                .mip_level_dimensions(self.level),
        );

        // Make sure the "offset" doesn't cause reads outside the texture
        if let Some(subregion) = subregion {
            if mip_level_region.is_larger_than(subregion) {
                return Err(MipLevelReadError::InvalidRegion());
            }
        }

        // Get the mip level subregion if the given one is None
        let subregion = subregion.unwrap_or(mip_level_region);

        // TODO: Actually handle reading here
        todo!();
    }

    // Write some pixels to the mip level region from the given source
    pub fn write(
        &mut self,
        src: &[<T::T as Texel>::Storage],
        subregion: Option<T::Region>,
    ) -> Result<(), MipLevelWriteError> {
        todo!()
    }

    // Copy a sub-region from another level into this level
    pub fn copy_subregion_from(
        &mut self,
        other: impl AsRef<MipLevelRef<'a, T>>,
        src_subregion: Option<T::Region>,
        dst_subregion: Option<T::Region>,
    ) -> Result<(), MipLevelCopyError> {
        todo!()
    }

    // Clear a region of the mip level to zero
    pub fn clear(
        &mut self,
        subregion: Option<T::Region>,
    ) -> Result<(), MipLevelClearError> {
        todo!()
    }

    // Fill the mip level region with a repeating value specified by "val"
    pub fn splat(
        &mut self,
        subregion: Option<T::Region>,
        val: <T::T as Texel>::Storage,
    ) -> Result<(), MipLevelWriteError> {
        todo!()
    }
}
