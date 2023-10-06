use std::sync::Arc;

use phobos::{Instance, PhysicalDevice, Device, DefaultAllocator, pool::ResourcePool, ExecutionManager, DebugMessenger, FrameManager, Surface};

/// Simple graphics resource that is shareable to other threads in case we
/// wish to multithread some CPU intensive application (like model/image loading)
pub struct Graphics {
    pub instance: Instance,
    pub physical_device: PhysicalDevice,
    pub device: Device,
    pub allocator: DefaultAllocator,
    pub pool: ResourcePool,
    pub exec: ExecutionManager,
    pub frame: Option<FrameManager>,
    pub surface: Option<Surface>,
    pub debug_messenger: Option<DebugMessenger>,
}