use crate::{Graphics, Region, Texel, TextureError, TextureMode, TextureUsage, UntypedTexel};

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
    fn new(
        graphics: &Graphics,
        texels: Texels<Self::T>,
        mode: TextureMode,
        usage: TextureUsage,
    ) -> Result<Self, TextureError> {
        // First of all, check if the format is supported for the supported usage and mode
        let UntypedTexel { 
            format,
            channels,
            element,
            total_bits,
            bits_per_channel
        } = <Self::T as Texel>::untyped();
        todo!()
    }
}
