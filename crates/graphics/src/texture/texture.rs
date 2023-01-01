use vulkan::{Allocation, vk};

use crate::{Graphics, Region, Texel, TextureMode, TextureUsage, UntypedTexel, Extent, MipLevelMut, MipLevelRef, TextureInitializationError, TextureMipLayerError};

// Predefined texel data
type Texels<'a, T> = &'a [<T as Texel>::Storage];

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
    ) -> Result<Self, TextureInitializationError> {
        let UntypedTexel { 
            format,
            channels,
            element,
            total_bits,
            bits_per_channel
        } = <Self::T as Texel>::untyped();

        // Make sure the number of texels matches up with the dimensions
        if dimensions.area() as usize != texels.len() {
            let extent = dimensions.as_vk_extent();
            return Err(TextureInitializationError::TexelDimensionsMismatch(
                texels.len(), extent.width, extent.height, extent.depth
            ));
        }

        // Create a staging buffer that contains the data 

        // Get the image type using the dimensionality
        let image_type = match <<Self::Region as Region>::E as Extent>::dimensionality() {
            1 => vk::ImageType::TYPE_1D,
            2 => vk::ImageType::TYPE_2D,
            3 => vk::ImageType::TYPE_3D,
            _ => panic!("1D, 2D, or 3D textures are the only supported types of textures")
        };

        // Pick the vulkan image usage flags
        let image_usage_flags = vk::ImageUsageFlags::empty();

        // Create the raw Vulkan image
        /*
        graphics.device().create_image(
            dimensions.as_vk_extent(),
            image_usage_flags,
            format,
            image_type,
            vk::TILIN,
            mip_levels,
            location,
            queue
        );
        */

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

    // Get the texture's usage
    fn usage(&self) -> TextureUsage;

    // Get immutable access to the internal allocation
    fn allocation(&self) -> &Allocation;

    // Get mutable access to the internal allocation
    fn allocation_mut(&mut self) -> &mut Allocation;

    // Get a single mip level from the texture, immutably
    fn mip(&self, level: u8) -> Result<MipLevelRef<Self>, TextureMipLayerError> {
        todo!()
    }

    // Get a single mip level from the texture, mutably (uses internal mutability pattern)
    fn mip_mut(&mut self, level: u8) -> Result<MipLevelMut<Self>, TextureMipLayerError> {
        todo!()
    }

    // Create a texture struct from it's raw components
    // This will simply create the texture struct, and it assumes
    // that the texture was already created externally
    unsafe fn from_raw_parts(
        image: vk::Image,
        whole_view: vk::ImageView,
        allocation: Allocation,
        dimensions: <Self::Region as Region>::E,
        usage: TextureUsage,
        mode: TextureMode,
        graphics: &Graphics,
    ) -> Self;
}
