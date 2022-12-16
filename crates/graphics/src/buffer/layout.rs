use vulkan::{vk, MemoryLocation};
use crate::{BufferMode, BufferUsage};

// Buffer internal layout that contains memory location of src buffer
// and it's staging buffer (if it has one)
#[derive(Debug)]
pub(super) struct BufferLayouts {
    pub src_buffer_memory_location: MemoryLocation,
    pub src_buffer_usage_flags: vk::BufferUsageFlags,

    // Types of staging buffers and if we should use them
    pub init_staging_buffer: bool,
    pub cached_staging_buffer: bool,
}

/*

MemoryLocation::GpuOnly => vk::MemoryPropertyFlags::DEVICE_LOCAL,
            MemoryLocation::CpuToGpu => {
                vk::MemoryPropertyFlags::HOST_VISIBLE
                    | vk::MemoryPropertyFlags::HOST_COHERENT
                    | vk::MemoryPropertyFlags::DEVICE_LOCAL
            }
            MemoryLocation::GpuToCpu => {
                vk::MemoryPropertyFlags::HOST_VISIBLE
                    | vk::MemoryPropertyFlags::HOST_COHERENT
                    | vk::MemoryPropertyFlags::HOST_CACHED
            }

*/

// Convert buffer mode and buffer usage to memory location and buffer usage flags
// and potential staging buffer memory location
// This will ignore invalid use cases
pub(super) fn find_optimal_layout(
    usage: BufferUsage,
    _type: u32,
) -> BufferLayouts {
    let BufferUsage {
        device_write,
        device_read,
        host_write,
        host_read,
    } = usage;

    // Check if there is device (hint)
    let device = device_read || device_write;

    // Check if there is host access
    let host = host_read || host_write;

    // Convert the transfer src/dst into the appropriate flags
    let transfer = if device_write { vk::BufferUsageFlags::TRANSFER_DST } else { vk::BufferUsageFlags::empty() }
        | if device_read { vk::BufferUsageFlags::TRANSFER_SRC } else { vk::BufferUsageFlags::empty() };

    // Map buffer type to usage flags
    let base = vk::BufferUsageFlags::from_raw(_type) | transfer;

    // hint_device_read or hint_device_write only, GpuOnly, init staging
    if device && !host {
        return BufferLayouts {
            src_buffer_memory_location: MemoryLocation::GpuOnly,
            src_buffer_usage_flags: vk::BufferUsageFlags::TRANSFER_DST
                | base,
            init_staging_buffer: true,
            cached_staging_buffer: false,
        };
    }

    // if host_read and hint_device_write, GPUToCPU
    if host_read
        && device_write
        && !host_write
        && !device_read
    {
        return BufferLayouts {
            src_buffer_memory_location: MemoryLocation::GpuToCpu,
            src_buffer_usage_flags: base,
            init_staging_buffer: false,
            cached_staging_buffer: false,
        };
    }

    // if host_write and hint_device_read, CPUToGPU
    if host_write
        && device_read
        && !host_read
        && !device_write
    {
        return BufferLayouts {
            src_buffer_memory_location: MemoryLocation::CpuToGpu,
            src_buffer_usage_flags: base,
            init_staging_buffer: false,
            cached_staging_buffer: false,
        };
    }

    // This always works
    BufferLayouts {
        src_buffer_memory_location: MemoryLocation::CpuToGpu,
        src_buffer_usage_flags: base,
        init_staging_buffer: false,
        cached_staging_buffer: false,
    }
}