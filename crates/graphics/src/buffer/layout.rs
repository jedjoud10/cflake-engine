use crate::{BufferMode, BufferUsage};

// What type of buffer should we create
#[derive(Debug)]
pub(super) enum UniqueBufferLayoutKind {
    DeviceLocal,
    CpuAccessible,
}

// Buffer internal layout that contains memory location of src buffer
// and it's staging buffer (if it has one)
#[derive(Debug)]
pub(super) struct BufferLayouts {
    // Base buffer that we must create
    pub src_usage: vulkano::buffer::BufferUsage,
    pub src_host_cached: bool,
    pub src_kind: UniqueBufferLayoutKind,

    // Potential initial staging buffer
    pub init_usage: Option<vulkano::buffer::BufferUsage>,
    
    // Potential continus staging buffer
    pub cached_usage: Option<vulkano::buffer::BufferUsage>,
    pub cached_host_cached: Option<bool>,
}

// Convert buffer mode and buffer usage to memory location and buffer usage flags
// and potential staging buffer memory location
// This will ignore invalid use cases
// (og location, cached staging buffer, init staging buffer)
pub(super) fn find_optimal_layout(
    _mode: BufferMode,
    usage: BufferUsage,
) -> BufferLayouts {
    let BufferUsage {
        hint_device_write,
        hint_device_read,
        permission_host_write,
        permission_host_read,
    } = usage;

    // Check if there is device (hint)
    let device = hint_device_read || hint_device_write;

    // Check if there is host access
    let host = permission_host_write || permission_host_read;

    // GPU, DeviceLocalBuffer,
    if device && !host {
        return BufferLayouts {
            src_usage: vulkano::buffer::BufferUsage {
                transfer_dst: true,
                ..Default::default()
            },
            src_host_cached: false,
            src_kind: UniqueBufferLayoutKind::DeviceLocal,
            init_usage: Some(vulkano::buffer::BufferUsage {
                transfer_src: true,
                ..Default::default()
            }),
            cached_usage: None,
            cached_host_cached: None,
        };
    }

    // CPU -> GPU, CpuAccessibleBuffer
    if permission_host_read
        && hint_device_write
        && !permission_host_write
        && !hint_device_read
    {
        return BufferLayouts {
            src_usage: vulkano::buffer::BufferUsage::default(),
            src_host_cached: false,
            src_kind: UniqueBufferLayoutKind::CpuAccessible,
            init_usage: None,
            cached_usage: None,
            cached_host_cached: None,
        };
    }

    // CPU -> GPU, CpuAccessibleBuffer
    if permission_host_write
        && hint_device_read
        && !permission_host_read
        && !hint_device_write
    {
        return BufferLayouts {
            src_usage: vulkano::buffer::BufferUsage::default(),
            src_host_cached: false,
            src_kind: UniqueBufferLayoutKind::CpuAccessible,
            init_usage: None,
            cached_usage: None,
            cached_host_cached: None,
        };
    }

    // Fallback, CpuAccessibleBuffer
    BufferLayouts {
        src_usage: vulkano::buffer::BufferUsage::default(),
        src_host_cached: false,
        src_kind: UniqueBufferLayoutKind::CpuAccessible,
        init_usage: None,
        cached_usage: None,
        cached_host_cached: None,
    }
}