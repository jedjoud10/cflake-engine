use ahash::AHashMap;
use dashmap::DashMap;
use parking_lot::Mutex;
use phobos::{Instance, PhysicalDevice, Device, DefaultAllocator, pool::ResourcePool, ExecutionManager, FrameManager, DebugMessenger};
use std::{hash::BuildHasherDefault, path::PathBuf, sync::Arc};
use thread_local::ThreadLocal;
use utils::Storage;

// Internnal graphics context that will eventually be wrapped within an Arc
pub(crate) struct InternalGraphics {
    pub(crate) physical_device: PhysicalDevice,
    pub(crate) debug_messenger: DebugMessenger,
    pub(crate) device: Device,
    pub(crate) allocator: DefaultAllocator,
    pub(crate) pool: ResourcePool,
    pub(crate) exec: ExecutionManager,
    pub(crate) instance: Instance,
}

// Graphical context that we will wrap around the phobos instance
// This context must be shareable between threads to allow for multithreading
#[derive(Clone)]
pub struct Graphics(pub(crate) Arc<InternalGraphics>);

impl Graphics {
}
