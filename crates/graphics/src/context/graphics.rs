use std::sync::Arc;

use phobos::{Instance, PhysicalDevice, Device, DefaultAllocator, pool::ResourcePool, ExecutionManager};

/// Simple graphics resource that is shareable to other threads in case we
/// wish to multithread some CPU intensive application (like model/image loading)
#[derive(Clone)]
pub struct Graphics {
    pub physical_device: Arc<PhysicalDevice>,
    pub device: Device,
    pub allocator: DefaultAllocator,
    pub pool: ResourcePool,
    pub exec: ExecutionManager,
}