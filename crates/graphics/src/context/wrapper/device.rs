use crate::{FrameRateLimit, GraphicSettings, WindowSettings, Instance, Surface, Adapter, Queues};
use ash::{
    extensions::{
        ext::DebugUtils,
    },
    vk::{
        self, DeviceCreateInfo, DeviceQueueCreateInfo,
        PhysicalDevice, PhysicalDeviceFeatures,
        PhysicalDeviceMemoryProperties, PhysicalDeviceProperties,
    },
    Entry,
};
use bytemuck::{Zeroable, Pod};
use gpu_allocator::{vulkan::{AllocationCreateDesc, Allocation, AllocatorCreateDesc, Allocator}, MemoryLocation};
use raw_window_handle::{HasRawWindowHandle, HasRawDisplayHandle, RawWindowHandle, RawDisplayHandle};
use std::{
    borrow::Cow,
    ffi::{c_void, CStr, CString},
};
use winit::{
    event_loop::EventLoop,
    window::{Fullscreen, WindowBuilder},
};

// Wrapper around a logical device
pub(crate) struct Device {
    pub(crate) physical_device: PhysicalDevice,
    pub(crate) device: ash::Device,
    pub(crate) allocator: gpu_allocator::vulkan::Allocator,
}

impl Device {
    pub(crate) unsafe fn destroy(mut self) {
        self.device.device_wait_idle().unwrap();
        self.device.destroy_device(None);
    }
}


// Create a logical Vulkan device using the queues
pub(crate) unsafe fn create_device(
    instance: &Instance,
    adapter: &Adapter,
    queues: &mut Queues,
    graphic_settings: &GraphicSettings,
) -> Device {
    // Create the queue create infos
    let mut create_infos = queues.families.iter().zip(queues.priorities.iter()).map(|(index, priorities)| {
        *DeviceQueueCreateInfo::builder()
            .queue_priorities(&priorities)
            .queue_family_index(*index)
    }).collect::<Vec<_>>();

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
    let device = instance.instance
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
    let debug_settings = gpu_allocator::AllocatorDebugSettings::default();

    // Create a memory allocator (gpu_allocator)
    let allocator = Allocator::new(&AllocatorCreateDesc {
        instance: instance.instance.clone(),
        device: device.clone(),
        physical_device: adapter.physical_device,
        debug_settings,
        buffer_device_address: false,
    }).unwrap();

    Device {
        physical_device: adapter.physical_device,
        device,
        allocator,
    }
}

// Create a single simple semaphore
pub(crate) unsafe fn create_semaphore(
    device: &Device,
) -> vk::Semaphore {
    device.device.create_semaphore(&Default::default(), None).unwrap()
}

// Create a single simple fence
pub(crate) unsafe fn create_fence(device: &Device) -> vk::Fence {
    device.device.create_fence(&Default::default(), None).unwrap()
}

// Create a buffer with the proper flags and size
pub(crate) unsafe fn create_buffer(device: &mut Device, size: u64, usage: vk::BufferUsageFlags) -> (vk::Buffer, Allocation) {
    // Setup vulkan info
    let vk_info = vk::BufferCreateInfo::builder()
        .size(size)
        .sharing_mode(vk::SharingMode::EXCLUSIVE)
        .usage(usage);

    // Create the buffer and fetch requirements
    let buffer = device.device.create_buffer(&vk_info, None).unwrap();
    let requirements = device.device.get_buffer_memory_requirements(buffer);

    // Create gpu-allocator allocation
    let allocation = device.allocator
        .allocate(&AllocationCreateDesc {
            name: "",
            requirements,
            location: MemoryLocation::CpuToGpu,
            linear: true, // Buffers are always linear
        }).unwrap();

    // Bind memory to the buffer
    unsafe { device.device.bind_buffer_memory(buffer, allocation.memory(), allocation.offset()).unwrap() };

    // Return the newly made buffer and it's allocation
    (buffer, allocation)
}
