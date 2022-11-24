use crate::{Adapter, GraphicSettings, Instance, Queues};
use ash::vk::{
    self, DeviceCreateInfo, DeviceQueueCreateInfo, PhysicalDevice,
};

use gpu_allocator::{
    vulkan::{
        Allocation, AllocationCreateDesc, Allocator,
        AllocatorCreateDesc,
    },
    MemoryLocation,
};
use parking_lot::Mutex;

// Wrapper around a logical device
pub(crate) struct Device {
    pub(crate) physical_device: PhysicalDevice,
    pub(crate) device: ash::Device,
    pub(crate) allocator: Mutex<gpu_allocator::vulkan::Allocator>,
}

impl Device {
    // Create a logical Vulkan device using the queues
    pub(crate) unsafe fn new(
        instance: &Instance,
        adapter: &Adapter,
        queues: &mut Queues,
        graphic_settings: &GraphicSettings,
    ) -> Device {
        // Create the queue create infos
        let create_infos = queues
            .families
            .iter()
            .map(|index| {
                *DeviceQueueCreateInfo::builder()
                    .queue_priorities(&[1.0])
                    .queue_family_index(index.family_index)
            })
                .collect::<Vec<_>>();

        // Create logical device create info
        let logical_device_extensions = graphic_settings
            .logical_device_extensions
            .iter()
            .map(|s| s.as_ptr())
            .collect::<Vec<_>>();
        let logical_device_create_info = DeviceCreateInfo::builder()
            .queue_create_infos(&create_infos)
            .enabled_extension_names(&logical_device_extensions)
                .enabled_features(&adapter.physical_device_features);

        // Create the logical device
        let device = instance
            .instance
            .create_device(
                adapter.physical_device,
                &logical_device_create_info,
                None,
            )
                .expect("Could not create the logical device");

        

        // Pick allocator debug settings
        #[cfg(debug_assertions)]
        let debug_settings = gpu_allocator::AllocatorDebugSettings {
            log_memory_information: true,
            log_leaks_on_shutdown: true,
            store_stack_traces: true,
            log_allocations: true,
            log_frees: true,
            log_stack_traces: true,
            };

        // No debugging
        #[cfg(not(debug_assertions))]
        let debug_settings =
                gpu_allocator::AllocatorDebugSettings::default();

        // Create a memory allocator (gpu_allocator)
        let allocator = Allocator::new(&AllocatorCreateDesc {
            instance: instance.instance.clone(),
            device: device.clone(),
            physical_device: adapter.physical_device,
            debug_settings,
            buffer_device_address: false,
        })
            .unwrap();

        // Le logical device
        let device = Device {
            physical_device: adapter.physical_device,
            device,
            allocator: Mutex::new(allocator),
            };

        // Finish the queue creation
        super::complete_queue_creation(&device, queues);
        device
    }

    // Create a single simple semaphore
    pub(crate) unsafe fn create_semaphore(&self) -> vk::Semaphore {
        self
            .device
            .create_semaphore(&Default::default(), None)
            .unwrap()
    }

    // Create a single simple fence
    pub(crate) unsafe fn create_fence(&self) -> vk::Fence {
        self
            .device
            .create_fence(&Default::default(), None)
            .unwrap()
    }

    // Create raw buffer with no memory
    pub(crate) unsafe fn create_buffer(
        &self,
        size: u64,
        usage: vk::BufferUsageFlags
    ) -> vk::Buffer {
        // Setup vulkan info
        let vk_info = vk::BufferCreateInfo::builder()
            .size(size)
            .flags(vk::BufferCreateFlags::empty())
            .sharing_mode(vk::SharingMode::EXCLUSIVE)
            .usage(usage);

        // Create the buffer and fetch requirements
        self.device.create_buffer(&vk_info, None).unwrap()
    }

    // Create the underlying memory for a buffer
    pub(crate) unsafe fn create_buffer_memory(
        &self,
        buffer: vk::Buffer,
        location: MemoryLocation,
    ) -> Allocation {
        // Get memory requirements
        let requirements = self.device.get_buffer_memory_requirements(buffer);

        // Create gpu-allocator allocation
        let allocation = self
            .allocator
            .lock()
            .allocate(&AllocationCreateDesc {
                name: "",
                requirements,
                location,
                linear: true, // Buffers are always linear
            })
            .unwrap();

        // Bind memory to the buffer
        unsafe {
            self
                .device
                .bind_buffer_memory(
                    buffer,
                    allocation.memory(),
                    allocation.offset(),
                )
                .unwrap()
        };
        allocation
    }

    // Destroy the logical device
    pub(crate) unsafe fn destroy(self) {
        self.device.device_wait_idle().unwrap();
        self.device.destroy_device(None);
    }
}


