use crate::vulkan::{gpu_allocator::MemoryLocation, vk};

// How exactly are we going to use the buffer?
#[derive(Default, Clone, Copy, PartialEq, Eq, Debug)]
pub enum BufferUsage {
    // The buffer would live on GPU memory
    // Example: Static Vertex Buffers, Static UBO
    // DEVICE_LOCAL
    GpuOnly,

    // Buffer would mainly be used for CPU -> GPU upload
    // Example: Dynamic Vertex Buffers
    // HOST_VISIBLE, HOST_COHERENT, DEVICE_LOCAL
    #[default]
    CpuToGpu,

    // Buffer would mainly be used for GPU -> CPU readback
    // Example: Voxel data generated from compute
    // HOST_VISIBLE, HOST_COHERENT, HOST_CACHED
    GpuToCpu,
}

// Convert buffer mode and buffer usage to memory location and buffer usage flags
// and potential staging buffer memory location
// This will ignore invalid use cases
pub(super) fn find_optimal_layout(
    usage: BufferUsage,
    _type: u32,
) -> (MemoryLocation, vk::BufferUsageFlags) {
    let location = match usage {
        BufferUsage::GpuOnly => MemoryLocation::GpuOnly,
        BufferUsage::CpuToGpu => MemoryLocation::CpuToGpu,
        BufferUsage::GpuToCpu => MemoryLocation::GpuToCpu,
    };

    // Map buffer type to usage flags
    let base = vk::BufferUsageFlags::from_raw(_type)
        | vk::BufferUsageFlags::TRANSFER_DST
        | vk::BufferUsageFlags::TRANSFER_SRC;

    (location, base)
}
