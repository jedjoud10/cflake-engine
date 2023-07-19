use std::{cell::RefCell, marker::PhantomData, ops::RangeBounds};
use utils::BitSet;

use crate::{
    Extent, Region, RenderTarget, Texel, Texture, TextureUsage, TextureViewDimension,
    ViewAsTargetError, ViewClearError, ViewCopyError, ViewReadError, ViewWriteError,
};

// The view settings that we should create for the texture
// These will be given to the texture as an array to allow many views to be created
#[derive(Clone, Copy, PartialEq, Eq, Hash)]
pub struct TextureViewSettings {
    pub base_mip_level: u32,
    pub mip_level_count: Option<u32>,
    pub base_array_layer: u32,
    pub array_layer_count: Option<u32>,
    pub dimension: TextureViewDimension,
}

impl TextureViewSettings {
    // Create a texture view setting that represents the whole region (all mips all layers)
    pub fn whole<R: Region>() -> Self {
        Self {
            base_mip_level: 0,
            mip_level_count: None,
            base_array_layer: 0,
            array_layer_count: None,
            dimension: R::view_dimension(),
        }
    }
}

// Given the whole region of a view and an optional subregion return a valid region
fn handle_optional_subregion<R: Region>(whole: R, optional: Option<R>) -> Option<R> {
    // Make sure the "offset" doesn't cause reads outside the texture
    if let Some(subregion) = optional {
        if whole.is_larger_than(subregion) {
            return None;
        }
    }

    // Get the mip level subregion if the given one is None
    return Some(optional.unwrap_or(whole));
}

pub fn read<T: Texture>(
    texture: &T,
    view: &wgpu::TextureView,
    settings: &TextureViewSettings,
    whole: Option<T::Region>,
    subregion: Option<T::Region>,
    dst: &mut [<T::T as Texel>::Storage],
) -> Result<(), ViewReadError> {
    /*
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
     */

    todo!()
}

pub fn write<T: Texture>(
    texture: &T,
    view: &wgpu::TextureView,
    settings: &TextureViewSettings,
    whole: Option<T::Region>,
    subregion: Option<T::Region>,
    src: &[<T::T as Texel>::Storage],
) -> Result<(), ViewWriteError> {
    // Nothing to write to
    if src.is_empty() {
        return Ok(());
    }

    // Make sure we can write to the texture
    if !texture.usage().contains(TextureUsage::WRITE) {
        return Err(ViewWriteError::NonWritable);
    }

    // Cannot write to multiple levels at once
    if settings.mip_level_count.unwrap_or(texture.levels()) != 1 {
        return Err(ViewWriteError::MultipleMipLevels);
    }

    // Get a proper subregion with the given opt subregion
    let Some(subregion) = handle_optional_subregion(
        whole.unwrap(),
        subregion,
    ) else {
        return Err(ViewWriteError::InvalidRegion);
    };

    // Write to the view and into the specified subregion
    crate::write_to_level::<T::T, T::Region>(
        subregion.origin(),
        subregion.extent(),
        src,
        texture.raw(),
        settings.base_mip_level,
        &texture.graphics(),
    );

    Ok(())
}

pub fn copy_subregion_from<T: Texture, O: Texture<T = T::T>>(
    src: &O,
    dst: &T,
    src_view: &wgpu::TextureView,
    src_settings: &TextureViewSettings,
    src_whole: O::Region,
    dst_view: &wgpu::TextureView,
    dst_settings: &TextureViewSettings,
    dst_whole: T::Region,
    src_subregion: Option<O::Region>,
    dst_subregion: Option<T::Region>,
) -> Result<(), ViewCopyError> {
    

    Ok(())
}

pub fn clear<T: Texture>(
    texture: &T,
    view: &wgpu::TextureView,
    settings: &TextureViewSettings,
    whole: T::Region,
    subregion: Option<T::Region>,
) -> Result<(), ViewClearError> {
    todo!()
    /*
    // Make sure we can write to the texture
    if texture.usage().contains(TextureUsage::WRITE) {
        return Err(ViewClearError::NonWritable);
    }

    // Get a proper subregion with the given opt subregion
    let Some(subregion) = handle_optional_subregion(
        texture,
        self.level,
        subregion
    ) else {
        return Err(ViewClearError::InvalidRegion);
    };

    todo!()
    */
}

pub fn splat<T: Texture>(
    texture: &T,
    view: &wgpu::TextureView,
    settings: &TextureViewSettings,
    whole: Option<T::Region>,
    subregion: Option<T::Region>,
    val: <T::T as Texel>::Storage,
) -> Result<(), ViewWriteError> {
    let volume = if let Some(whole) = whole {
        let region = subregion.unwrap_or(whole);
        <T::Region as Region>::volume(region.extent()) as usize
    } else {
        0
    };

    let texels = vec![val; volume];
    write(texture, view, settings, whole, subregion, &texels)
}

// Singular texture view that might contain multiple layers / mips
pub struct TextureViewRef<'a, T: Texture> {
    pub(crate) texture: &'a T,
    pub(crate) view: &'a wgpu::TextureView,
    pub(crate) settings: &'a TextureViewSettings,
}

impl<'a, T: Texture> TextureViewRef<'a, T> {
    // Get the underlying texture
    pub fn texture(&self) -> &T {
        self.texture
    }

    // Get the underlying wgpu view
    pub fn raw(&self) -> &wgpu::TextureView {
        &self.view
    }

    // Get the view's dimensions (returns none if we are accessing multiple mips)
    pub fn dimensions(&self) -> Option<<T::Region as Region>::E> {
        (self.levels() == 1).then(|| {
            self.texture
                .dimensions()
                .mip_level_dimensions(self.settings.base_mip_level)
        })
    }

    // Get the view's region (returns none if we are accessing multiple mips)
    pub fn region(&self) -> Option<T::Region> {
        self.dimensions()
            .map(|d| <T::Region as Region>::from_extent(d))
    }

    // Get the number of visible levels in this view
    pub fn levels(&self) -> u32 {
        self.settings
            .mip_level_count
            .unwrap_or(self.texture.levels())
    }

    // Get the number of visible layers in this view
    pub fn layers(&self) -> u32 {
        self.settings
            .array_layer_count
            .unwrap_or(self.texture.layers())
    }
}

impl<'a, T: Texture> From<TextureViewMut<'a, T>> for TextureViewRef<'a, T> {
    fn from(value: TextureViewMut<'a, T>) -> Self {
        TextureViewRef {
            texture: value.texture,
            view: value.view,
            settings: value.settings,
        }
    }
}

// Singular mutable texture view that might contain multiple layers / mips
pub struct TextureViewMut<'a, T: Texture> {
    pub(crate) texture: &'a T,
    pub(crate) view: &'a wgpu::TextureView,
    pub(crate) settings: &'a TextureViewSettings,
}

impl<'a, T: Texture> TextureViewMut<'a, T> {
    // Get the underlying texture
    pub fn texture(&self) -> &T {
        self.texture
    }

    // Get the underlying wgpu view
    pub fn raw(&self) -> &wgpu::TextureView {
        &self.view
    }

    // Get the view's dimensions (returns none if we are accessing multiple mips)
    pub fn dimensions(&self) -> Option<<T::Region as Region>::E> {
        (self.levels() == 1).then(|| {
            let dims = self
                .texture
                .dimensions()
                .mip_level_dimensions(self.settings.base_mip_level);
            let (x, y, _) = dims.decompose().into_tuple();
            let z = self.layers();
            <<T::Region as Region>::E as Extent>::new(x, y, z)
        })
    }

    // Get the view's region (returns none if we are accessing multiple mips)
    pub fn region(&self) -> Option<T::Region> {
        self.dimensions()
            .map(|d| <T::Region as Region>::from_extent(d))
    }

    // Get the number of visible levels in this view
    pub fn levels(&self) -> u32 {
        self.settings
            .mip_level_count
            .unwrap_or(self.texture.levels())
    }

    // Get the number of visible layers in this view
    pub fn layers(&self) -> u32 {
        self.settings
            .array_layer_count
            .unwrap_or(self.texture.layers())
    }

    // Write some texels to the texture view
    pub fn write(
        &mut self,
        subregion: Option<T::Region>,
        src: &[<T::T as Texel>::Storage],
    ) -> Result<(), ViewWriteError> {
        write(
            self.texture,
            self.view,
            self.settings,
            self.region(),
            subregion,
            src,
        )
    }

    // Fill the view region with a repeating value specified by "val"
    pub fn splat(
        &mut self,
        subregion: Option<T::Region>,
        val: <T::T as Texel>::Storage,
    ) -> Result<(), ViewWriteError> {
        splat(
            self.texture,
            self.view,
            self.settings,
            self.region(),
            subregion,
            val,
        )
    }

    // Copy a view from another texture view
    pub fn copy_from<'b>(
        &mut self,
        other: impl Into<TextureViewRef<'b, T>>
    ) -> Result<(), ViewCopyError> {
        let other: TextureViewRef<'b, T> = other.into();

        // Make sure we can use the texture as copy src
        if !other.texture.usage().contains(TextureUsage::COPY_SRC) {
            return Err(ViewCopyError::NonCopySrc);
        }

        // Make sure we can use the texture as copy dst
        if !self.texture.usage().contains(TextureUsage::COPY_DST) {
            return Err(ViewCopyError::NonCopyDst);
        }

        

        Ok(())

        /*
        // Get a proper subregion with the given opt subregion for dst
        let Some(dst_subregion) = handle_optional_subregion(
            self.texture,
            self.level,
            dst_subregion
        ) else {
            return Err(ViewCopyError::InvalidSrcRegion);
        };

        // Get a proper subregion with the given opt subregion for src
        let Some(src_subregion) = handle_optional_subregion(
            other.texture,
            other.level,
            src_subregion
        ) else {
            return Err(ViewCopyError::InvalidDstRegion);
        };
        */

        /*
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
            _ => return Err(ViewCopyError::IncompatibleMultiLayerTextures),
        };
        */
    }

    // Try to use the texture view as a renderable target.
    // This will fail if the texture isn't supported as render target
    // or if the view's dimensions don't correspond to a 2D image
    pub fn as_render_target(&mut self) -> Result<RenderTarget<T::T>, ViewAsTargetError> {
        if !self.texture.usage().contains(TextureUsage::TARGET) {
            return Err(ViewAsTargetError::MissingTargetUsage);
        }

        if self.levels() > 1 {
            return Err(ViewAsTargetError::ViewMultipleMips);
        }

        if self.layers() > 1 {
            return Err(ViewAsTargetError::RegionIsNot2D);
        }

        Ok(RenderTarget {
            _phantom: PhantomData,
            view: &self.view,
        })
    }
}
