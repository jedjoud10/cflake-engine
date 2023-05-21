use std::{cell::Cell, marker::PhantomData, num::NonZeroU8, ops::DerefMut};

use bytemuck::Zeroable;
use rayon::prelude::{IntoParallelIterator, ParallelIterator};

use super::{Region, Texture};
use crate::{
    ColorTexel, Conversion, Extent, LayeredOrigin, MipLevelClearError, MipLevelCopyError,
    MipLevelReadError, MipLevelWriteError, Origin, RenderTarget, Texel, TextureAsTargetError,
    TextureMipLevelError, TextureSamplerError, TextureUsage,
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
            Self::Manual { mips } => Self::Manual { mips: mips.clone() },
        }
    }
}

// Calculate mip levels based on the given color data and size
// Returns None if the texture isn't a power of two texture
pub fn generate_mip_map<T: ColorTexel, R: Region>(
    base: &[T::Storage],
    extent: R::E,
) -> Option<Vec<Vec<T::Storage>>> {
    // Convert a xyz value to an index (texel)
    fn xyz_to_index(location: vek::Vec3<usize>, extent: vek::Extent3<usize>) -> usize {
        location.x + location.y * extent.w + location.z * (extent.w * extent.h)
    }

    // Create manual mip maps for this texture
    let dimension = <R as Region>::view_dimension();
    let name = utils::pretty_type_name::<T>();
    let levels = R::levels(extent)?.get() as u32;
    log::debug!("Creating mip-data (max = {levels}) for imported texture {dimension:?}, <{name}>");

    // Iterate over the levels and fill them up
    // (like how ceddy weddy fills me up inside >.<)
    let map = (0..(levels - 1))
        .into_par_iter()
        .map(|i| {
            // Pre-allocate a vector that will contain the downscaled texels
            let temp = extent.mip_level_dimensions(i as u8);
            let downscaled = extent.mip_level_dimensions(i as u8 + 1);

            let mut texels: Vec<<T as Texel>::Storage> =
                vec![<T::Storage as Zeroable>::zeroed(); R::volume(downscaled) as usize];

            // Get the original and downscaled sizes
            let original = temp.decompose();
            let new = downscaled.decompose();

            // Division factor is either 2, 4, or 8 (based on dims)
            let factor = match dimension {
                wgpu::TextureViewDimension::D1 => 2,
                wgpu::TextureViewDimension::D2 => 4,
                wgpu::TextureViewDimension::D2Array => 4,
                wgpu::TextureViewDimension::Cube => 4,
                wgpu::TextureViewDimension::CubeArray => 4,
                wgpu::TextureViewDimension::D3 => 8,
            };

            // Nous devons pas prendre une moyenne de l'axe Z si nous utilisons une ArrayTexture2D
            let divide = match R::view_dimension() {
                wgpu::TextureViewDimension::D1 => vek::Vec3::new(2usize, 1, 1),
                wgpu::TextureViewDimension::D2Array => vek::Vec3::new(2, 2, 1),
                wgpu::TextureViewDimension::D2 => vek::Vec3::new(2, 2, 1),
                wgpu::TextureViewDimension::D3 => vek::Vec3::new(2, 2, 2),
                wgpu::TextureViewDimension::CubeArray => todo!(),
                wgpu::TextureViewDimension::Cube => todo!(),
            };

            // Write to the downscaled texels
            for ox in 0..original.w {
                for oy in 0..original.h {
                    for oz in 0..original.d {
                        // Get the current texel value
                        let texel = base[xyz_to_index(
                            vek::Vec3::new(ox, oy, oz).as_::<usize>() * divide.map(|x| x.pow(i)),
                            extent.decompose().as_::<usize>(),
                        )];

                        // La division est vraiment importante pour qu'on evite un overflow
                        let texel = T::divide(texel, factor as f32);

                        // Get the destination texel value
                        let dst = &mut texels[xyz_to_index(
                            vek::Vec3::new(ox, oy, oz).as_::<usize>() / divide,
                            new.as_::<usize>(),
                        )];

                        // Sum to the destination
                        *dst += texel;
                    }
                }
            }

            // Return the texels
            texels
        })
        .collect::<Vec<_>>();

    Some(map)
}

// Given the texture dimensions and the given optional sub-region,return a valid sub-region
// Returns None if the given subregion is greater than the miup level region (so it's invalid)
fn handle_optional_subregion<T: Texture>(
    texture: &T,
    level: u8,
    optional: Option<T::Region>,
) -> Option<T::Region> {
    // Get the region for this mip level
    let mip_level_region =
        <T::Region as Region>::from_extent(texture.dimensions().mip_level_dimensions(level));

    // Make sure the "offset" doesn't cause reads outside the texture
    if let Some(subregion) = optional {
        if mip_level_region.is_larger_than(subregion) {
            return None;
        }
    }

    // Get the mip level subregion if the given one is None
    return Some(optional.unwrap_or(mip_level_region));
}

// Collection of multiple immutable mip levels
pub struct MipLevelsRef<'a, T: Texture> {
    pub(super) texture: &'a T,
}

impl<'a, T: Texture> MipLevelsRef<'a, T> {
    // Get the number of mip levels contained within the texture
    pub fn len(&self) -> u32 {
        self.texture.levels()
    }

    // Borrow a mip-level from the mip collection immutably
    pub fn level(&'a self, level: u8) -> Result<MipLevelRef<'a, T>, TextureMipLevelError> {
        let range = self.len() as u8;
        if level > range {
            Ok(MipLevelRef {
                texture: self.texture,
                level: level,
                borrowed: None,
            })
        } else {
            Err(TextureMipLevelError::OutOfRange(level, range))
        }
    }
}

// Collection of multiple mutable mip levels
pub struct MipLevelsMut<'a, T: Texture> {
    pub(super) texture: &'a mut T,
    pub(super) mutated: Cell<u32>,
    pub(crate) borrowed: Cell<u32>,
}

impl<'a, T: Texture> MipLevelsMut<'a, T> {
    // Get the number of mip levels contained within the texture
    pub fn len(&self) -> u32 {
        self.texture.levels()
    }

    // Borrow a mip-level from the mip collection immutably
    pub fn level(&'a self, level: u8) -> Result<MipLevelRef<'a, T>, TextureMipLevelError> {
        let range = self.len() as u8;
        if level > range {
            Ok(MipLevelRef {
                texture: self.texture,
                level: level,
                borrowed: Some(&self.borrowed),
            })
        } else {
            Err(TextureMipLevelError::OutOfRange(level, range))
        }
    }

    // Borrow a mip-level from the mip collection mutably
    pub fn level_mut(&'a self, level: u8) -> Result<MipLevelMut<'a, T>, TextureMipLevelError> {
        let range = self.len() as u8;
        if level < range {
            Ok(MipLevelMut {
                texture: self.texture,
                level: level,
                mutated: &self.mutated,
            })
        } else {
            Err(TextureMipLevelError::OutOfRange(level, range))
        }
    }
}

// An immutable mip level that we can use to read from the texture
pub struct MipLevelRef<'a, T: Texture> {
    pub(super) texture: &'a T,
    pub(super) level: u8,
    pub(crate) borrowed: Option<&'a Cell<u32>>,
}

// Helper methods
impl<'a, T: Texture> MipLevelRef<'a, T> {
    // Get the underlying texture
    pub fn texture(&self) -> &T {
        self.texture
    }

    // Get a view for this mip
    pub fn view(&self) -> Option<&wgpu::TextureView> {
        crate::get_specific_view(self.texture, None, Some(self.level as u32))
    }

    // Get the view of a specific layer of the current mip
    pub fn layer_view(&self, layer: u32) -> Option<&wgpu::TextureView>
    where
        <T::Region as Region>::O: LayeredOrigin,
    {
        crate::get_specific_view(self.texture, Some(layer), Some(self.level as u32))
    }

    // Get the mip level of the current level
    pub fn level(&self) -> u8 {
        self.level
    }

    // Get the mip level's dimensions
    pub fn dimensions(&self) -> <T::Region as Region>::E {
        self.texture.dimensions().mip_level_dimensions(self.level)
    }

    // Get the mip level's region
    pub fn region(&self) -> T::Region {
        T::Region::from_extent(self.dimensions())
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

        // Get a proper subregion with the given opt subregion
        let Some(subregion) = handle_optional_subregion(
            self.texture,
            self.level,
            subregion
        ) else {
            return Err(MipLevelReadError::InvalidRegion);
        };

        // Read from the mip level and from the specified sub-region
        super::read_from_level::<T::T, T::Region>(
            subregion.origin(),
            subregion.extent(),
            dst,
            self.texture.raw(),
            self.level as u32,
            &self.texture.graphics(),
        );

        Ok(())
    }
}

impl<'a, T: Texture> Drop for MipLevelRef<'a, T> {
    fn drop(&mut self) {
        if let Some(borrowed) = self.borrowed {
            let copied = borrowed.get();
            borrowed.set(copied & !(1u32 << self.level));
        }
    }
}

// A mutable mip level that we can use to write to the texture
pub struct MipLevelMut<'a, T: Texture> {
    pub(crate) texture: &'a T,
    pub(crate) level: u8,
    pub(super) mutated: &'a Cell<u32>,
}

// Helper methods
impl<'a, T: Texture> MipLevelMut<'a, T> {
    // Get the underlying texture
    pub fn texture(&self) -> &T {
        self.texture
    }

    // Get a view for this mip
    pub fn view(&self) -> Option<&wgpu::TextureView> {
        crate::get_specific_view(self.texture, None, Some(self.level as u32))
    }

    // Get the view of a specific layer of the current mip
    pub fn layer_view(&self, layer: u32) -> Option<&wgpu::TextureView>
    where
        <T::Region as Region>::O: LayeredOrigin,
    {
        crate::get_specific_view(self.texture, Some(layer), Some(self.level as u32))
    }

    // Get the mip level of the current level
    pub fn level(&self) -> u8 {
        self.level
    }

    // Get the mip level's dimensions
    pub fn dimensions(&self) -> <T::Region as Region>::E {
        self.texture.dimensions().mip_level_dimensions(self.level)
    }

    // Get the mip level's region
    pub fn region(&self) -> T::Region {
        T::Region::from_extent(self.dimensions())
    }

    // Try to get a render target so we can render to this one mip level as a whole
    // Returns an Error if the texture is not renderable
    pub fn as_render_target(&mut self) -> Result<RenderTarget<T::T>, TextureAsTargetError> {
        if !self.texture().usage().contains(TextureUsage::TARGET) {
            return Err(TextureAsTargetError::MissingTargetUsage);
        }

        Ok(RenderTarget {
            _phantom: PhantomData,
            view: self.view().unwrap(),
        })
    }

    // Try to get a render target so we can render to a specific layer of this mip level
    // Returns an Error if the texture is not renderable or if the layer specified is invalid
    pub fn layer_as_render_target(
        &mut self,
        layer: u32,
    ) -> Result<RenderTarget<T::T>, TextureAsTargetError>
    where
        <T::Region as Region>::O: LayeredOrigin,
    {
        if !self.texture().usage().contains(TextureUsage::TARGET) {
            return Err(TextureAsTargetError::MissingTargetUsage);
        }

        Ok(RenderTarget {
            _phantom: PhantomData,
            view: self.layer_view(layer).unwrap(),
        })
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

        // Get a proper subregion with the given opt subregion
        let Some(subregion) = handle_optional_subregion(
            self.texture,
            self.level,
            subregion
        ) else {
            return Err(MipLevelReadError::InvalidRegion);
        };

        // Read from the mip level and from the specified sub-region
        super::read_from_level::<T::T, T::Region>(
            subregion.origin(),
            subregion.extent(),
            dst,
            self.texture.raw(),
            self.level as u32,
            &self.texture.graphics(),
        );

        Ok(())
    }

    // Write some pixels to the mip level region from the given source
    pub fn write(
        &mut self,
        src: &[<T::T as Texel>::Storage],
        subregion: Option<T::Region>,
    ) -> Result<(), MipLevelWriteError> {
        // Nothing to write to
        if src.is_empty() {
            return Ok(());
        }

        // Make sure we can write to the texture
        if !self.texture.usage().contains(TextureUsage::WRITE) {
            return Err(MipLevelWriteError::NonWritable);
        }

        // Get a proper subregion with the given opt subregion
        let Some(subregion) = handle_optional_subregion(
            self.texture,
            self.level,
            subregion
        ) else {
            return Err(MipLevelWriteError::InvalidRegion);
        };

        // Write to the mip level level and into the specified sub-region
        crate::write_to_level::<T::T, T::Region>(
            subregion.origin(),
            subregion.extent(),
            src,
            &self.texture.raw(),
            self.level as u32,
            &self.texture.graphics(),
        );

        Ok(())
    }

    // Copy a sub-region from another level into this level
    pub fn copy_subregion_from<O: Texture<T = T::T>>(
        &mut self,
        other: MipLevelRef<'a, O>,
        src_subregion: Option<O::Region>,
        dst_subregion: Option<T::Region>,
    ) -> Result<(), MipLevelCopyError> {
        // Make sure we can use the texture as copy src
        if !other.texture.usage().contains(TextureUsage::COPY_SRC) {
            return Err(MipLevelCopyError::NonCopySrc);
        }

        // Make sure we can use the texture as copy dst
        if !self.texture.usage().contains(TextureUsage::COPY_DST) {
            return Err(MipLevelCopyError::NonCopyDst);
        }

        // Get a proper subregion with the given opt subregion for dst
        let Some(dst_subregion) = handle_optional_subregion(
            self.texture,
            self.level,
            dst_subregion
        ) else {
            return Err(MipLevelCopyError::InvalidSrcRegion);
        };

        // Get a proper subregion with the given opt subregion for src
        let Some(src_subregion) = handle_optional_subregion(
            other.texture,
            other.level,
            src_subregion
        ) else {
            return Err(MipLevelCopyError::InvalidDstRegion);
        };

        // Make sure that the layers are compatible
        match (
            <O::Region as Region>::view_dimension(),
            <T::Region as Region>::view_dimension(),
        ) {
            // Copy from 2D array to self
            (wgpu::TextureViewDimension::D2Array, wgpu::TextureViewDimension::Cube)
                if other.texture.layers() == 6 => {}
            //(wgpu::TextureViewDimension::D2Array, wgpu::TextureViewDimension::CubeArray) => todo!(),

            // Copy from Cube to self
            (wgpu::TextureViewDimension::Cube, wgpu::TextureViewDimension::D2Array)
                if self.texture.layers() == 6 => {}
            //(wgpu::TextureViewDimension::Cube, wgpu::TextureViewDimension::CubeArray) => todo!(),

            /*
            // Copy from CubeArray to self
            (wgpu::TextureViewDimension::CubeArray, wgpu::TextureViewDimension::D2Array) => todo!(),
            (wgpu::TextureViewDimension::CubeArray, wgpu::TextureViewDimension::Cube) => todo!(),
            */
            (x, y) if x == y => (),
            _ => return Err(MipLevelCopyError::IncompatibleMultiLayerTextures),
        };

        todo!();

        Ok(())
    }

    // Clear a region of the mip level to zero
    pub fn clear(&mut self, subregion: Option<T::Region>) -> Result<(), MipLevelClearError> {
        // Make sure we can write to the texture
        if !self.texture.usage().contains(TextureUsage::WRITE) {
            return Err(MipLevelClearError::NonWritable);
        }

        // Get a proper subregion with the given opt subregion
        let Some(subregion) = handle_optional_subregion(
            self.texture,
            self.level,
            subregion
        ) else {
            return Err(MipLevelClearError::InvalidRegion);
        };

        todo!()
    }

    // Fill the mip level region with a repeating value specified by "val"
    pub fn splat(
        &mut self,
        subregion: Option<T::Region>,
        val: <T::T as Texel>::Storage,
    ) -> Result<(), MipLevelWriteError> {
        // Get the region for this mip level
        let mip_level_region = <T::Region as Region>::from_extent(
            self.texture.dimensions().mip_level_dimensions(self.level),
        );

        // Get the mip level subregion if the given one is None
        let region = subregion.unwrap_or(mip_level_region);
        let volume = <T::Region as Region>::volume(region.extent()) as usize;
        let texels = vec![val; volume];
        self.write(&texels, subregion)
    }
}

impl<'a, T: Texture> Drop for MipLevelMut<'a, T> {
    fn drop(&mut self) {
        let copied = self.mutated.get();
        self.mutated.set(copied & !(1u32 << self.level));
    }
}
