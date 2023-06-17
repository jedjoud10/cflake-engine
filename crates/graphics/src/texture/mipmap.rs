use std::{cell::Cell, marker::PhantomData, num::NonZeroU8, ops::DerefMut};

use bytemuck::Zeroable;
use rayon::prelude::{IntoParallelIterator, ParallelIterator};

use super::{Region, Texture};
use crate::{
    ColorTexel, Conversion, Extent, LayeredOrigin, ViewClearError, ViewCopyError,
    ViewReadError, ViewWriteError, Origin, RenderTarget, Texel, ViewAsTargetError,
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


