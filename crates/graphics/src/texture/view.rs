use std::{ops::RangeBounds, marker::PhantomData, cell::RefCell};
use utils::BitSet;

use crate::{TextureViewDimension, Texture, Region, Extent, ViewReadError, Texel, ViewWriteError, ViewCopyError, ViewClearError, RenderTarget, ViewAsTargetError, TextureUsage};

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

pub fn read<T: Texture>(
    texture: &T,
    view: &wgpu::TextureView,
    whole: T::Region,
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
    whole: T::Region,
    subregion: Option<T::Region>,
    src: &[<T::T as Texel>::Storage],
) -> Result<(), ViewWriteError> {
    /*
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
    */
    todo!()
}

pub fn copy_subregion_from<T: Texture, O: Texture<T = T::T>>(
    src: &O,
    dst: &T,
    src_view: &wgpu::TextureView,
    src_whole: O::Region,
    dst_view: &wgpu::TextureView,
    dst_whole: T::Region,
    src_subregion: Option<O::Region>,
    dst_subregion: Option<T::Region>,
) -> Result<(), ViewCopyError> {
    todo!()
    /*
    // Make sure we can use the texture as copy src
    if !other.texture.usage().contains(TextureUsage::COPY_SRC) {
        return Err(ViewCopyError::NonCopySrc);
    }

    // Make sure we can use the texture as copy dst
    if !self.texture.usage().contains(TextureUsage::COPY_DST) {
        return Err(ViewCopyError::NonCopyDst);
    }

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

    todo!();

    Ok(())
    */
}

pub fn clear<T: Texture>(
    texture: &T,
    view: &wgpu::TextureView,
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
    whole: T::Region,
    subregion: Option<T::Region>,
    val: <T::T as Texel>::Storage,
) -> Result<(), ViewWriteError> {
    let region = subregion.unwrap_or(whole);
    let volume = <T::Region as Region>::volume(region.extent()) as usize;
    let texels = vec![val; volume];
    //self.write(&texels, subregion)
    Ok(())
}

// Singular texture view that might contain multiple layers / mips
pub struct TextureViewRef<'a, T: Texture> {
    pub(crate) texture: &'a T,
    pub(crate) view: &'a wgpu::TextureView,
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

    // Get the view's dimensions
    pub fn dimensions(&self) -> <T::Region as Region>::E {
        todo!()
    }

    // Get the view's region
    pub fn region(&self) -> T::Region {
        todo!()
    }
}

// Singular mutable texture view that might contain multiple layers / mips
pub struct TextureViewMut<'a, T: Texture> {
    pub(crate) texture: &'a T,
    pub(crate) view: &'a wgpu::TextureView,
}

impl<'a, T: Texture> TextureViewMut<'a, T> {
    // Try to use the texture view as a renderable target.
    // This will fail if the texture isn't supported as render target 
    // or if the view's dimensions don't correspond to a 2D image
    pub fn as_render_target(&mut self) -> Result<RenderTarget<T::T>, ViewAsTargetError> {
        if !self.texture.usage().contains(TextureUsage::TARGET) {
            return Err(ViewAsTargetError::MissingTargetUsage);
        }

        /*
        TODO: This shit
        if self.levels() > 1 {
            return Err(ViewAsTargetError::ViewMultipleMips);
        }

        if !self.region().can_render_to_mip() {
            return Err(ViewAsTargetError::RegionIsNot2D);
        }
        */

        Ok(RenderTarget {
            _phantom: PhantomData,
            view: &self.view,
        })
    }
}

/*

// A mutable mip level that we can use to write to the texture
pub struct MipLevelMut<'a, T: Texture> {
    pub(crate) texture: &'a T,
    pub(crate) level: u8,
    pub(super) mutated: &'a Cell<u32>,
}

// Helper methods
impl<'a, T: Texture> MipLevelMut<'a, T> {
    
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

    
}

impl<'a, T: Texture> Drop for MipLevelMut<'a, T> {
    fn drop(&mut self) {
        let copied = self.mutated.get();
        self.mutated.set(copied & !(1u32 << self.level));
    }
}
*/