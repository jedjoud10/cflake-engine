use ash::vk;
use gpu_allocator::MemoryLocation;

use crate::{BufferMode, BufferUsage};

// Buffer internal layout that contains memory location of src buffer
// and it's staging buffer (if it has one)
#[derive(Debug)]
pub(super) struct BufferLayouts {
    pub src_buffer_memory_location: MemoryLocation,
    pub src_buffer_usage_flags: vk::BufferUsageFlags,

    // Used during init times
    pub init_staging_buffer_memory_location: Option<MemoryLocation>,
    pub init_staging_buffer_usage_flags: Option<vk::BufferUsageFlags>,

    // Used for staging buffer that is stored alongside the original buffer
    pub cached_staging_buffer_memory_location: Option<MemoryLocation>,
    pub cached_staging_buffer_usage_flags:
        Option<vk::BufferUsageFlags>,
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
// (og location, cached staging buffer, init staging buffer)
pub(super) fn find_optimal_layout(
    _mode: BufferMode,
    usage: BufferUsage,
    _type: u32,
) -> BufferLayouts {
    let BufferUsage {
        hint_device_write,
        hint_device_read,
        host_write,
        host_read,
    } = usage;

    // Check if there is device (hint)
    let device = hint_device_read || hint_device_write;

    // Check if there is host access
    let host = host_read || host_write;

    // Map buffer type to usage flags
    let base = match _type {
        0 => vk::BufferUsageFlags::VERTEX_BUFFER,
        1 => vk::BufferUsageFlags::INDEX_BUFFER,
        2 => vk::BufferUsageFlags::STORAGE_BUFFER,
        3 => vk::BufferUsageFlags::UNIFORM_BUFFER,
        4 => vk::BufferUsageFlags::INDIRECT_BUFFER,
        _ => panic!(),
    };

    // hint_device_read or hint_device_write only, GpuOnly, init staging
    if device && !host {
        return BufferLayouts {
            src_buffer_memory_location: MemoryLocation::GpuOnly,
            src_buffer_usage_flags: vk::BufferUsageFlags::TRANSFER_DST
                | base,
            init_staging_buffer_memory_location: Some(
                MemoryLocation::CpuToGpu,
            ),
            init_staging_buffer_usage_flags: Some(
                vk::BufferUsageFlags::TRANSFER_SRC,
            ),
            cached_staging_buffer_memory_location: None,
            cached_staging_buffer_usage_flags: None,
        };
    }

    // if host_read and hint_device_write, GPUToCPU
    if host_read
        && hint_device_write
        && !host_write
        && !hint_device_read
    {
        return BufferLayouts {
            src_buffer_memory_location: MemoryLocation::GpuToCpu,
            src_buffer_usage_flags: base,
            init_staging_buffer_memory_location: None,
            init_staging_buffer_usage_flags: None,
            cached_staging_buffer_memory_location: None,
            cached_staging_buffer_usage_flags: None,
        };
    }

    // if host_write and hint_device_read, CPUToGPU
    if host_write
        && hint_device_read
        && !host_read
        && !hint_device_write
    {
        return BufferLayouts {
            src_buffer_memory_location: MemoryLocation::CpuToGpu,
            src_buffer_usage_flags: base,
            init_staging_buffer_memory_location: None,
            init_staging_buffer_usage_flags: None,
            cached_staging_buffer_memory_location: None,
            cached_staging_buffer_usage_flags: None,
        };
    }

    // This always works
    BufferLayouts {
        src_buffer_memory_location: MemoryLocation::CpuToGpu,
        src_buffer_usage_flags: base,
        init_staging_buffer_memory_location: None,
        init_staging_buffer_usage_flags: None,
        cached_staging_buffer_memory_location: None,
        cached_staging_buffer_usage_flags: None,
    }
}
