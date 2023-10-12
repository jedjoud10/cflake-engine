use std::sync::Arc;
use phobos::{Instance, PhysicalDevice, Device, DefaultAllocator, pool::ResourcePool, ExecutionManager, DebugMessenger, FrameManager, Surface};


/// Simple graphics resource that is shareable to other threads in case we
/// wish to multithread some CPU intensive application (like model/image loading)
#[derive(Clone)]
pub struct Graphics {
    pub pool: ResourcePool,
    pub allocator: DefaultAllocator,
    pub exec: ExecutionManager,
    pub device: Device,
    pub physical_device: Arc<PhysicalDevice>,
    pub debug_messenger: Option<Arc<DebugMessenger>>,
    pub instance: Arc<Instance>,
}