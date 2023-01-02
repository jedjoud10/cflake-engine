use vulkan::{Allocation, vk};

use crate::{Graphics, Region, Texel, TextureMode, TextureUsage, UntypedTexel, Extent, MipLevelMut, MipLevelRef, TextureInitializationError, TextureMipLayerError, Sampler, TextureSamplerError};

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
            bits_per_channel
        } = <Self::T as Texel>::untyped();

        // Make sure the number of texels matches up with the dimensions
        if dimensions.area() as usize != texels.len() {
            let extent = dimensions.as_vk_extent();
            return Err(TextureInitializationError::TexelDimensionsMismatch(
                texels.len(), extent.width, extent.height, extent.depth
            ));
        }

        // Calculate how many bytes we should allocate for this texture
        let bits = u64::from(dimensions.area())
         * u64::from(bits_per_channel)
         * u64::from(channels.count());
        let bytes = bits / 8;

        // Create a staging buffer that contains the texel data
        let device = graphics.device();
        let queue = graphics.queue();
        let mut block = unsafe {
            device.staging_pool().lock(device, queue, bytes)
        };

        // Fill the staging buffer with the corresponding texel data
        let slice = bytemuck::cast_slice_mut::<u8, <Self::T as Texel>::Storage>(block.mapped_slice_mut());
        slice.copy_from_slice(texels);

        // Get the image type using the dimensionality
        let image_type = match <<Self::Region as Region>::E as Extent>::dimensionality() {
            1 => vk::ImageType::TYPE_1D,
            2 => vk::ImageType::TYPE_2D,
            3 => vk::ImageType::TYPE_3D,
            _ => panic!("1D, 2D, or 3D textures are the only supported types of textures")
        };

        // Pick the vulkan image usage flags
        let image_usage_flags = vk::ImageUsageFlags::TRANSFER_DST | vk::ImageUsageFlags::SAMPLED;

        // Create the raw Vulkan image
        let (image, allocation) = unsafe {
            graphics.device().create_image(
                dimensions.as_vk_extent(),
                image_usage_flags,
                format,
                image_type,
                1,
                1,
                vk::SampleCountFlags::TYPE_1,
                vulkan::MemoryLocation::GpuOnly,
                queue
            )
        };

        // Optimal image layout for our specific use
        let dst_image_layout = vk::ImageLayout::SHADER_READ_ONLY_OPTIMAL;
        let dst_access_mask = vk::AccessFlags::SHADER_READ;

        // Convert image layouts, copy, and then convert to optimal one
        unsafe {
            let mut recorder = queue.acquire(device);

            // Image whole subresource range (TODO: Implement mipmapping
            let subresource_range = vk::ImageSubresourceRange::builder()
                .aspect_mask(vk::ImageAspectFlags::COLOR)
                .base_mip_level(0)
                .base_array_layer(0)
                .layer_count(1)
                .level_count(1);

            // TBH Idk what is the difference between ImageSubresourceLayers and  ImageSubresourceRange
            let subresource_layers = vk::ImageSubresourceLayers::builder()
                .aspect_mask(vk::ImageAspectFlags::COLOR)
                .mip_level(0)
                .base_array_layer(0)
                .layer_count(1);

            // Convert the image layout to TRANSFER_DST first
            let image_barrier_to_transfer_dst = vk::ImageMemoryBarrier::builder()
                .old_layout(vk::ImageLayout::UNDEFINED)
                .new_layout(vk::ImageLayout::TRANSFER_DST_OPTIMAL)
                .src_access_mask(vk::AccessFlags::empty())
                .dst_access_mask(vk::AccessFlags::TRANSFER_WRITE)
                .subresource_range(*subresource_range)
                .image(image);

            // Copy the buffer data into the texture
            let copy = vk::BufferImageCopy::builder()
                .buffer_offset(block.offset())
                .buffer_row_length(0)
                .buffer_image_height(0)
                .image_extent(dimensions.as_vk_extent())
                .image_offset(vk::Offset3D::default())
                .image_subresource(*subresource_layers);

            // Convert back to the usage defined layout 
            let image_barrier_to_optimal_dst = vk::ImageMemoryBarrier::builder()
                .old_layout(vk::ImageLayout::TRANSFER_DST_OPTIMAL)
                .new_layout(dst_image_layout)
                .src_access_mask(vk::AccessFlags::TRANSFER_WRITE)
                .dst_access_mask(dst_access_mask)
                .subresource_range(*subresource_range)
                .image(image);

            // Copy buffer data to the image
            recorder.cmd_image_memory_barrier(*image_barrier_to_transfer_dst);
            recorder.cmd_copy_buffer_to_image(block.buffer(), image, vk::ImageLayout::TRANSFER_DST_OPTIMAL, &[*copy]);
            recorder.cmd_image_memory_barrier(*image_barrier_to_optimal_dst);

            recorder.immediate_submit();
        }

        Ok(unsafe {
            Self::from_raw_parts(
                image,
                vk::ImageView::null(),
                allocation,
                dimensions,
                usage,
                mode,
                graphics,
            )
        })
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

    // Try to get a sampler for this texture so we can read from it within shaders 
    fn try_fetch_sampler(&self) -> Result<Sampler<Self>, TextureSamplerError> {
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
