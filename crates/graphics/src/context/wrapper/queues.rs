use crate::{FrameRateLimit, GraphicSettings, WindowSettings, Instance, Surface, Adapter};
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

// Queues and their families that will be used by the logical device
pub(crate) struct Queues {
    pub(crate) families: Vec<u32>,
    pub(crate) priorities: Vec<Vec<f32>>,
}

impl Queues {
    pub(super) unsafe fn destroy(mut self) {
    }
}

// Get the required queues from a logical device
pub(crate) unsafe fn create_queues(
    adapter: &Adapter,
    surface: &Surface,
    instance: &Instance,
) -> Queues {
    let families = instance.instance.get_physical_device_queue_family_properties(
        adapter.physical_device,
    );

    // Get the present queue family
    let present =pick_queue_family(
        &families,
        &surface,
        &adapter,
        true,
        vk::QueueFlags::empty()
    );

    // Get the graphics queue family
    let graphics = pick_queue_family(
        &families,
        &surface,
        &adapter,
        false,
        vk::QueueFlags::GRAPHICS
    );

    // Convert to vector
    let mut families = vec![present, graphics];
    families.sort_unstable();
    families.dedup();

    // Create one queue per family for now
    let priorities = (0..families.len())
        .into_iter()
        .map(|_| vec![1.0f32])
        .collect::<Vec<_>>();

    Queues {
        families,
        priorities,
    }
}

// Find a queue that supports the specific flags
pub(super) unsafe fn pick_queue_family(
    queue_families: &[vk::QueueFamilyProperties],
    surface: &Surface, 
    adapter: &Adapter,
    supports_presenting: bool,
    flags: vk::QueueFlags,
) -> u32 {
    queue_families
        .iter()
        .enumerate()
        .position(|(i, props)| {
            // Check if the queue family supporsts the flags
            let flags = props.queue_flags.contains(flags);

            // If the queue we must fetch must support presenting, fetch the physical device properties
            let presenting = !supports_presenting
                || surface.surface_loader
                    .get_physical_device_surface_support(
                        adapter.physical_device,
                        i as u32,
                        surface.surface,
                    )
                    .unwrap();

            flags && presenting
        })
        .expect("Could not find the graphics queue") as u32
}
