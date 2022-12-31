use crate::{Graphics, Region, Texel, TextureError, TextureMode, TextureUsage, UntypedTexel, Extent, MipLevelMut, MipLevelRef};

// Possibly predefined texel data
type Texels<'a, T> = Option<&'a [<T as Texel>::Storage]>;

// A texture is an abstraction over Vulkan images to allow us to access/modify them with ease
// A texture is a container of multiple texels (like pixels, but for textures) that are stored on the GPU
// This trait is implemented for all variants of textures (1D, 2D, 3D, Layered)
pub trait Texture: Sized {
    // Texel region (position + extent)
    type Region: Region;

    // Texel layout that we will use internally
    type T: Texel;

    // Create a new texture with some possibly predefined data
    fn from_texels(
        graphics: &Graphics,
        texels: Texels<Self::T>,
        dimensions: <Self::Region as Region>::E,
        mode: TextureMode,
        usage: TextureUsage,
    ) -> Result<Self, TextureError> {
        let UntypedTexel { 
            format,
            channels,
            element,
            total_bits,
            bits_per_channel
        } = <Self::T as Texel>::untyped();

        log::info!("{:?}", format);
        todo!();
    }

    // Get the texture's region (origin state is default)
    fn region(&self) -> Self::Region {
        Self::Region::with_extent(self.dimensions())
    }

    // Checks if we can access a region of the texture
    fn is_region_valid(&self, region: Self::Region) -> bool {
        let extent =
            <Self::Region as Region>::extent_from_origin(region.origin()) + region.extent();
        self.dimensions().is_larger_than(extent)
    }

    // Get the texture's dimensions
    fn dimensions(&self) -> <Self::Region as Region>::E;

    // Get the texture's mode
    fn mode(&self) -> TextureMode;

    // Get the number of axii that this texture uses
    fn dimensionality(&self) -> usize;
    
    // Get the number of layer that this texture uses
    fn layers(&self) -> usize;

    // Get the texture's usage
    fn usage(&self) -> TextureUsage;

    // Get a single mip level from the texture, immutably
    fn mip(&self, level: u8) -> Result<MipLevelRef<Self>, TextureError> {
        todo!()
    }

    // Get a single mip level from the texture, mutably (uses internal mutability pattern)
    fn mip_mut(&mut self, level: u8) -> Result<MipLevelMut<Self>, TextureError> {
        todo!()
    }
}
