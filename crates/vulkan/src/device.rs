use crate::{
    required_features, Adapter, Instance, Queue, StagingBlock,
    StagingPool,
};
use ahash::AHashMap;
use ash::vk::{self, DeviceCreateInfo, DeviceQueueCreateInfo};
use dashmap::DashMap;
use gpu_allocator::vulkan::{
    Allocation, AllocationCreateDesc, Allocator, AllocatorCreateDesc,
};
use parking_lot::{MappedMutexGuard, Mutex, MutexGuard};

// This is a logical device that can run multiple commands and that can create Vulkan objects
// TODO: Check objects that are currently being used by the GPU?
pub struct Device {
    device: ash::Device,
    allocator: Mutex<Option<Allocator>>,
    glsl_spirv_translator: shaderc::Compiler,
    pipeline_cache: vk::PipelineCache,
    staging: StagingPool,
}

impl Device {
    // Create a new logical device from the physical adapter
    pub unsafe fn new(
        instance: &Instance,
        adapter: &Adapter,
    ) -> Device {
        // Let one queue family handle everything
        let family = Queue::pick_queue_family(
            adapter,
            true,
            vk::QueueFlags::GRAPHICS
                | vk::QueueFlags::COMPUTE
                | vk::QueueFlags::TRANSFER,
        );

        // Create the queue create infos
        let create_infos = (std::iter::once(family))
            .map(|family| {
                *DeviceQueueCreateInfo::builder()
                    .queue_priorities(&[1.0])
                    .queue_family_index(family as u32)
            })
            .collect::<Vec<_>>();

        // Create logical device create info
        let required_device_extensions =
            super::required_device_extensions();
        let logical_device_extensions = required_device_extensions
            .iter()
            .map(|s| s.as_ptr())
            .collect::<Vec<_>>();

        // Get the features that we must enable
        let mut adapter_features = required_features();

        // Create the device create info
        let logical_device_create_info = DeviceCreateInfo::builder()
            .queue_create_infos(&create_infos)
            .enabled_extension_names(&logical_device_extensions)
            .enabled_features(&adapter_features.features)
            .push_next(&mut adapter_features.features11)
            .push_next(&mut adapter_features.features12)
            .push_next(&mut adapter_features.features13);

        // Create the logical device
        let device = instance
            .raw()
            .create_device(
                adapter.physical_device(),
                &logical_device_create_info,
                None,
            )
            .expect("Could not create the logical device");
        log::debug!("Created the Vulkan device successfully");

        // Pick allocator debug settings
        #[cfg(debug_assertions)]
        let debug_settings = gpu_allocator::AllocatorDebugSettings {
            log_memory_information: false,
            log_leaks_on_shutdown: true,
            store_stack_traces: false,
            log_allocations: true,
            log_frees: false,
            log_stack_traces: false,
        };

        // No debugging
        #[cfg(not(debug_assertions))]
        let debug_settings =
            gpu_allocator::AllocatorDebugSettings::default();

        // Create a memory allocator (gpu_allocator)
        let allocator = Allocator::new(&AllocatorCreateDesc {
            instance: instance.raw().clone(),
            device: device.clone(),
            physical_device: adapter.physical_device(),
            debug_settings,
            buffer_device_address: false,
        })
        .unwrap();
        log::debug!("Created the Vulkan memory allocator");

        // Drop the cstrings
        drop(required_device_extensions);

        // Create pipeline cache to optimize pipeline creation
        let pipeline_cache = device
            .create_pipeline_cache(
                &vk::PipelineCacheCreateInfo::default(),
                None,
            )
            .unwrap();

        Device {
            device,
            allocator: Mutex::new(Some(allocator)),
            glsl_spirv_translator: shaderc::Compiler::new().unwrap(),
            pipeline_cache,
            staging: StagingPool::new(),
        }
    }

    // Get the underlying raw device
    pub fn raw(&self) -> &ash::Device {
        &self.device
    }

    // Lock the GPU allocator mutably
    pub fn allocator(&self) -> MappedMutexGuard<Allocator> {
        MutexGuard::map(self.allocator.lock(), |f| {
            f.as_mut().unwrap()
        })
    }

    // Get the underlying staging pool
    pub fn staging_pool(&self) -> &StagingPool {
        &self.staging
    }

    // Wait until the device executes all the code submitted to the GPU
    pub fn wait(&self) {
        unsafe {
            self.device.device_wait_idle().unwrap();
        }
    }

    // Destroy the logical device
    pub unsafe fn destroy(&self) {
        self.wait();
        self.device
            .destroy_pipeline_cache(self.pipeline_cache, None);
        self.staging.deallocate(self);
        self.allocator.lock().take().unwrap();
        self.device.destroy_device(None);
    }
}

// Synchronization
impl Device {
    // Create a single simple semaphore
    pub unsafe fn create_semaphore(&self) -> vk::Semaphore {
        self.device
            .create_semaphore(&Default::default(), None)
            .unwrap()
    }

    // Destroy a semaphore
    pub unsafe fn destroy_semaphore(&self, semaphore: vk::Semaphore) {
        self.device.destroy_semaphore(semaphore, None);
    }

    // Create a single simple fence
    pub unsafe fn create_fence(&self) -> vk::Fence {
        self.device.create_fence(&Default::default(), None).unwrap()
    }

    // Destroy a fence
    pub unsafe fn destroy_fence(&self, fence: vk::Fence) {
        self.device.destroy_fence(fence, None);
    }
}

// Pipelines and shader modules
impl Device {
    // Translate some GLSL shader code to SPIRV
    pub unsafe fn translate_glsl_spirv(
        &self,
        code: &str,
        file_name: &str,
        entry_point: &str,
        kind: shaderc::ShaderKind,
    ) -> Vec<u32> {
        // TODO: Make use of "additional_options" to manually add the include callback instead of using shitty processor
        let binary_result = self
            .glsl_spirv_translator
            .compile_into_spirv(
                code,
                kind,
                file_name,
                entry_point,
                None,
            )
            .unwrap();
        binary_result.as_binary().to_owned()
    }

    // Create a new shader module from SPIRV byte code
    pub unsafe fn compile_shader_module(
        &self,
        bytecode: &[u32],
    ) -> vk::ShaderModule {
        let create_info =
            vk::ShaderModuleCreateInfo::builder().code(bytecode);
        self.raw().create_shader_module(&create_info, None).unwrap()
    }

    // Create a new graphics pipeline based on the given info
    pub unsafe fn create_graphics_pipeline(
        &self,
        create_info: vk::GraphicsPipelineCreateInfo,
    ) -> vk::Pipeline {
        self.raw()
            .create_graphics_pipelines(
                self.pipeline_cache,
                &[create_info],
                None,
            )
            .unwrap()[0]
    }

    // Create a new compute pipeline based on the given info
    pub unsafe fn create_compute_pipeline(
        &self,
        create_info: vk::ComputePipelineCreateInfo,
    ) -> vk::Pipeline {
        self.raw()
            .create_compute_pipelines(
                self.pipeline_cache,
                &[create_info],
                None,
            )
            .unwrap()[0]
    }

    // Create a framebuffer and a renderpass combined
    // (since we store them in the same wrapper struct anyways)
    // Note: This creates the frame buffer with the IMAGELESS flag
    pub unsafe fn create_render_pass_framebuffer(
        &self,
        attachments: &[vk::AttachmentDescription],
        subpasses: &[vk::SubpassDescription],
        dependencies: &[vk::SubpassDependency],
        attachment_image_infos: &[vk::FramebufferAttachmentImageInfo],
        extent: vek::Extent2<u32>,
        layers: u32,
    ) -> (vk::RenderPass, vk::Framebuffer) {
        // Create the render pass first
        let render_pass_create_info = vk::RenderPassCreateInfo::builder()
            .dependencies(dependencies)
            .attachments(attachments)
            .subpasses(subpasses);
        let render_pass = self.raw().create_render_pass(&render_pass_create_info, None).unwrap();

        // Imageless attachment image infos
        let mut frame_buffer_attachments_create_info = vk::FramebufferAttachmentsCreateInfo::builder()
            .attachment_image_infos(attachment_image_infos);
        
        // Create null image views since the framebuffer is imagless
        let count = attachment_image_infos.len();
        let image_views = vec![vk::ImageView::null(); count];

        // Create info for the framebuffer
        let framebuffer_create_info = vk::FramebufferCreateInfo::builder()
            .attachments(&image_views)
            .width(extent.w)
            .height(extent.h)
            .render_pass(render_pass)
            .layers(layers)
            .flags(vk::FramebufferCreateFlags::IMAGELESS)
            .push_next(&mut frame_buffer_attachments_create_info);
        let framebuffer = self.raw().create_framebuffer(&framebuffer_create_info, None).unwrap();

        // Combine and return
        (render_pass, framebuffer)
    }

    // Destroy a specific render pass and a framebuffer
    pub unsafe fn destroy_render_pass_and_framebuffer(
        &self,
        render_pass: vk::RenderPass,
        framebuffer: vk::Framebuffer,
    ) {
        self.raw().destroy_render_pass(render_pass, None);
        self.raw().destroy_framebuffer(framebuffer, None);
    }

    // Destroy a specific pipeline
    pub unsafe fn destroy_pipeline(&self, pipeline: vk::Pipeline) {
        self.raw().destroy_pipeline(pipeline, None);
    }

    // Destroy a specific shader module
    pub unsafe fn destroy_shader_module(
        &self,
        module: vk::ShaderModule,
    ) {
        self.raw().destroy_shader_module(module, None);
    }
}

// Samplers
impl Device {
    // Create an image sampler
    pub unsafe fn create_sampler(
        &self,
        format: vk::Format,
        filter: vk::Filter,
        address_mode: vk::SamplerAddressMode,
        max_anisotropy: Option<f32>,
        border_color: vk::BorderColor,
        custom_border_color: vk::ClearColorValue,
        mipmap_mode: Option<(f32, f32, f32, vk::SamplerMipmapMode)>,
    ) -> vk::Sampler {
        let builder = vk::SamplerCreateInfo::builder()
            .mag_filter(filter)
            .min_filter(filter)
            .address_mode_u(address_mode)
            .address_mode_v(address_mode)
            .address_mode_w(address_mode);

        let builder = if let Some(max_anisotropy) = max_anisotropy {
            builder
                .anisotropy_enable(true)
                .max_anisotropy(max_anisotropy)
        } else {
            builder
        };

        let builder = if let Some((min_lod, max_lod, lod_bias, mode)) = mipmap_mode {
            builder
                .mipmap_mode(mode)
                .min_lod(min_lod)
                .max_lod(max_lod)
                .mip_lod_bias(lod_bias)
        } else {
            builder
        };

        let mut next = vk::SamplerCustomBorderColorCreateInfoEXT::builder()
            .custom_border_color(custom_border_color)
            .format(format);

        let builder = if border_color == vk::BorderColor::FLOAT_CUSTOM_EXT ||
        border_color == vk::BorderColor::INT_CUSTOM_EXT {
            builder
                .push_next(&mut next)
        } else {
            builder    
        };
        
        self.raw().create_sampler(&builder, None).unwrap()
    }

    // Destroy an image sampler
    pub unsafe fn destroy_sampler(
        &self,
        sampler: vk::Sampler
    ) {
        self.raw().destroy_sampler(sampler, None);
    }
}

// Buffers and images
impl Device {
    // Create a raw buffer and allocate the needed memory for it
    pub unsafe fn create_buffer(
        &self,
        size: u64,
        usage: vk::BufferUsageFlags,
        location: gpu_allocator::MemoryLocation,
        queue: &Queue,
    ) -> (vk::Buffer, Allocation) {
        // Setup vulkan info
        let arr = [queue.qfi];
        let vk_info = vk::BufferCreateInfo::builder()
            .size(size)
            .flags(vk::BufferCreateFlags::empty())
            .sharing_mode(vk::SharingMode::EXCLUSIVE)
            .queue_family_indices(&arr)
            .usage(usage);

        // Create the buffer and fetch requirements
        log::debug!(
            "Creating buffer with size {} and usage {:?}",
            size,
            usage
        );
        let buffer =
            self.device.create_buffer(&vk_info, None).unwrap();

        // Get memory requirements
        log::debug!("Creating buffer memory for buffer {:?}", buffer);
        let requirements =
            self.device.get_buffer_memory_requirements(buffer);

        // Create gpu-allocator allocation
        let allocation = self
            .allocator()
            .allocate(&AllocationCreateDesc {
                name: "buffer",
                requirements,
                location,
                linear: true,
            })
            .unwrap();

        // Bind memory to the buffer
        unsafe {
            self.device
                .bind_buffer_memory(
                    buffer,
                    allocation.memory(),
                    allocation.offset(),
                )
                .unwrap()
        };

        // Create the tuple and return it
        (buffer, allocation)
    }

    // Create a raw image and allocate the needed memory for it
    pub unsafe fn create_image(
        &self,
        extent: vk::Extent3D,
        usage: vk::ImageUsageFlags,
        format: vk::Format,
        _type: vk::ImageType,
        mip_levels: u32,
        layers: u32,
        samples: vk::SampleCountFlags,
        location: gpu_allocator::MemoryLocation,
        queue: &Queue,
    ) -> (vk::Image, Allocation) {
        // Setup vulkan info
        let arr = [queue.qfi];
        let vk_info = vk::ImageCreateInfo::builder()
            .extent(extent)
            .flags(vk::ImageCreateFlags::empty())
            .sharing_mode(vk::SharingMode::EXCLUSIVE)
            .queue_family_indices(&arr)
            .usage(usage)
            .array_layers(layers)
            .format(format)
            .samples(samples)
            .image_type(_type)
            .mip_levels(mip_levels)
            .tiling(vk::ImageTiling::OPTIMAL);

        // Create the image and fetch requirements
        log::debug!(
            "Creating image with extent {:?} and usage {:?}",
            extent,
            usage
        );
        let image =
            self.device.create_image(&vk_info, None).unwrap();

        // Get memory requirements
        log::debug!("Creating image memory for image {:?}", image);
        let requirements =
            self.device.get_image_memory_requirements(image);

        // Create gpu-allocator allocation
        let allocation = self
            .allocator()
            .allocate(&AllocationCreateDesc {
                name: "image",
                requirements,
                location,
                linear: false,
            })
            .unwrap();

        // Bind memory to the image
        unsafe {
            self.device
                .bind_image_memory(
                    image,
                    allocation.memory(),
                    allocation.offset(),
                )
                .unwrap()
        };

        // Create the tuple and return it
        (image, allocation)
    }

    // Free a buffer and it's allocation
    pub unsafe fn destroy_buffer(
        &self,
        buffer: vk::Buffer,
        allocation: Allocation,
    ) {
        // Deallocate the underlying memory
        log::debug!(
            "Freeing allocation with mapped ptr: {:?}",
            allocation.mapped_ptr()
        );
        self.allocator().free(allocation).unwrap();

        // Delete the Vulkan buffer
        log::debug!("Freeing buffer {:?}", buffer);
        self.device.destroy_buffer(buffer, None);
    }    

    // Free an image and it's allocation
    pub unsafe fn destroy_image(
        &self,
        image: vk::Image,
        allocation: Allocation,
    ) {
        // Deallocate the underlying memory
        log::debug!(
            "Freeing allocation with mapped ptr: {:?}",
            allocation.mapped_ptr()
        );
        self.allocator().free(allocation).unwrap();

        // Delete the Vulkan image
        log::debug!("Freeing image {:?}", image);
        self.device.destroy_image(image, None);
    }
}

// Buffer and image views
impl Device{
    // Create a new image view for an image
    pub unsafe fn create_image_view(
        &self,
        flags: vk::ImageViewCreateFlags,
        image: vk::Image,
        view_type: vk::ImageViewType,
        format: vk::Format,
        components: vk::ComponentMapping,
        subresource_range: vk::ImageSubresourceRange,
    ) -> vk::ImageView {
        let create_info = vk::ImageViewCreateInfo::builder()
            .components(components)
            .subresource_range(subresource_range)
            .format(format)
            .view_type(view_type)
            .image(image)
            .flags(flags);
        
        self.raw().create_image_view(&create_info, None).unwrap()
    }

    // Destroy an image view
    pub unsafe fn destroy_image_view(
        &self,
        view: vk::ImageView
    ) {
        self.raw().destroy_image_view(view, None);
    }
}