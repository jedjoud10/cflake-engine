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
        queues: &Queues,
        device_extensions: Vec<CString>,
    ) -> Device {
        // Create the queue create infos
        /*
        let create_infos = queues
            .fa
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
            .enabled_features(&adapter.features);

        // Create the logical device
        let device = instance
            .instance
            .create_device(
                adapter.raw,
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
            physical_device: adapter.raw,
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
