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
        self.allocator.lock().take().unwrap();
        self.device.destroy_device(None);
    }
}

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

impl Device {
    // Translate some GLSL shader code to SPIRV
    pub unsafe fn translate_glsl_spirv(
        &self,
        code: &str,
        file_name: &str,
        entry_point: &str,
        kind: shaderc::ShaderKind,
    ) -> Vec<u32> {
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
                vk::PipelineCache::null(),
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
                vk::PipelineCache::null(),
                &[create_info],
                None,
            )
            .unwrap()[0]
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
                name: "",
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
        let buffer = buffer;
        log::debug!("Freeing buffer {:?}", buffer);
        self.device.destroy_buffer(buffer, None);
    }
}
