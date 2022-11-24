use super::{FrameRateLimit, GraphicSettings, WindowSettings};
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


// Plain old data type internally used by buffers and other types
pub trait Content:
    Zeroable + Pod + Clone + Copy + Sync + Send + 'static
{
}
impl<T: Clone + Copy + Sync + Send + Zeroable + Pod + 'static> Content
    for T
{
}