use ash::vk;
use parking_lot::Mutex;

use crate::Device;

use super::family::Family;

// A single command pool abstraction
// We technically should have one pool per thread
pub struct Pool {
    // Raw vulkan command pool
    pub(crate) alloc: vk::CommandPool,
    
    // All the command buffers allocated in this command pool
    pub(crate) buffers: Mutex<Vec<vk::CommandBuffer>>,
}