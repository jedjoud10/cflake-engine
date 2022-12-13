use crate::{Adapter, Instance, Queue, required_features};
use ash::vk::{self, DeviceCreateInfo, DeviceQueueCreateInfo};
use gpu_allocator::vulkan::{Allocator, AllocatorCreateDesc, Allocation, AllocationCreateDesc};
use parking_lot::{Mutex, MappedMutexGuard, MutexGuard};

// This is a logical device that can run multiple commands and that can create Vulkan objects
pub struct Device {
    device: ash::Device,
    allocator: Mutex<Option<Allocator>>,
}

impl Device {
    // Create a new logical device from the physical adapter
    pub unsafe fn new(
        instance: &Instance,
        adapter: &Adapter,
    ) -> Device {
        // Get the graphics and present queue family
        let family = crate::Queue::pick_queue_family(
            &adapter.queue_family_properties,
            adapter,
            true,
            vk::QueueFlags::GRAPHICS,
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
        let enabled_features = required_features();
            let logical_device_create_info = DeviceCreateInfo::builder()
            .queue_create_infos(&create_infos)
            .enabled_extension_names(&logical_device_extensions)
            .enabled_features(&enabled_features);

        // Create the logical device
        let device = instance
            .instance
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
            instance: instance.instance.clone(),
            device: device.clone(),
            physical_device: adapter.physical_device(),
            debug_settings,
            buffer_device_address: false,
        })
        .unwrap();
        log::debug!(
            "Created the Vulkan memory allocator"
        );

        // Drop the cstrings
        drop(required_device_extensions);

        // Le logical device

        Device {
            device,
            allocator: Mutex::new(Some(allocator)),
        }
    }

    // Get the underlying raw device
    pub fn raw(&self) -> &ash::Device {
        &self.device
    }

    // Lock the GPU allocator mutably
    pub fn allocator(&self) -> MappedMutexGuard<Allocator> {
        MutexGuard::map(self.allocator.lock(), |f| f.as_mut().unwrap())
    }

    // Destroy the logical device
    pub unsafe fn destroy(&self) {
        self.allocator.lock().take().unwrap();
        self.device.device_wait_idle().unwrap();
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

    // Create a single simple fence
    pub unsafe fn create_fence(&self) -> vk::Fence {
        self.device.create_fence(&Default::default(), None).unwrap()
    }
}


impl Device {
    // Create raw buffer with no memory
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

    // Get the device address of a buffer
    pub unsafe fn buffer_device_address(
        &self,
        buffer: vk::Buffer,
    ) -> vk::DeviceAddress {
        let builder =
            vk::BufferDeviceAddressInfo::builder().buffer(buffer);
        self.device.get_buffer_device_address(&builder)
    }

    // Free a buffer and it's allocation
    pub unsafe fn destroy_buffer(
        &self,
        buffer: vk::Buffer,
        allocation: Allocation,
    ) {
        // Deallocate the underlying memory
        log::debug!(
            "Freeing allocation {:?}",
            allocation.mapped_ptr()
        );
        self.allocator().free(allocation).unwrap();

        // Delete the Vulkan buffer
        let buffer = buffer;
        log::debug!("Freeing buffer {:?}", buffer);
        self.device.destroy_buffer(buffer, None);
    }
}
