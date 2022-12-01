use std::ffi::CString;

use crate::{Adapter, Instance, Queues};
use ash::vk::{self, DeviceCreateInfo, DeviceQueueCreateInfo};

use gpu_allocator::{
    vulkan::{
        Allocation, AllocationCreateDesc, Allocator,
        AllocatorCreateDesc,
    },
    MemoryLocation,
};
use parking_lot::Mutex;
use utils::ThreadPool;

// Wrapper around a Vulkan logical device
pub struct Device {
    // Ash logical device
    pub(crate) device: ash::Device,

    // Memory allocator that will suballocate when needed
    pub(crate) allocator: Mutex<gpu_allocator::vulkan::Allocator>,
}

impl Device {
    // Create a logical Vulkan device using the queues
    pub unsafe fn new(
        instance: &Instance,
        adapter: &Adapter,
        queues: &mut Queues,
        device_extensions: Vec<CString>,
    ) -> Device {
        /*
        // Create the queue create infos
        let create_infos = queues
            .families()
            .map(|family| {
                *DeviceQueueCreateInfo::builder()
                    .queue_priorities(&[1.0])
                    .queue_family_index(family.index())
            })
            .collect::<Vec<_>>();
        */

        // Create logical device create info
        let logical_device_extensions = device_extensions
            .iter()
            .map(|s| s.as_ptr())
            .collect::<Vec<_>>();
        let logical_device_create_info = DeviceCreateInfo::builder()
            //.queue_create_infos(&create_infos)
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
            store_stack_traces: false,
            log_allocations: true,
            log_frees: true,
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
            physical_device: adapter.physical_device,
            debug_settings,
            buffer_device_address: false,
        })
        .unwrap();

        // Le logical device
        let device = Device {
            device,
            allocator: Mutex::new(allocator),
        };

        device
    }

    // Destroy the logical device
    pub unsafe fn destroy(self) {
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

    // Create raw buffer with no memory
    pub unsafe fn create_buffer(
        &self,
        size: u64,
        usage: vk::BufferUsageFlags,
        queues: &Queues,
    ) -> vk::Buffer {
        // Setup vulkan info
        //let graphics = queues.family(FamilyType::Graphics);
        //let indices = [graphics.index()];
        /*
        let vk_info = vk::BufferCreateInfo::builder()
            .size(size)
            .flags(vk::BufferCreateFlags::empty())
            .sharing_mode(vk::SharingMode::EXCLUSIVE)
            .queue_family_indices(&indices)
            .usage(usage);
        

        // Create the buffer and fetch requirements
        log::debug!(
            "Creating buffer with size {} and usage {:?}",
            size,
            usage
        );
        self.device.create_buffer(&vk_info, None).unwrap()
        */
        todo!()
    }

    // Create the underlying memory for a buffer
    pub unsafe fn create_buffer_memory(
        &self,
        buffer: vk::Buffer,
        location: MemoryLocation,
    ) -> Allocation {
        // Get memory requirements
        log::debug!("Creating buffer memory for buffer {:?}", buffer);
        let requirements =
            self.device.get_buffer_memory_requirements(buffer);

        // Create gpu-allocator allocation
        let allocation = self
            .allocator
            .lock()
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
        allocation
    }

    // Get the device address of a buffer
    pub unsafe fn buffer_device_address(
        &self,
        buffer: vk::Buffer
    ) -> vk::DeviceAddress {
        let builder = vk::BufferDeviceAddressInfo::builder().buffer(buffer);
        self.device.get_buffer_device_address(&*builder)
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
        self.allocator.lock().free(allocation).unwrap();

        // Delete the Vulkan buffer
        let buffer = buffer;
        log::debug!("Freeing buffer {:?}", buffer);
        self.device.destroy_buffer(buffer, None);
    }
}
