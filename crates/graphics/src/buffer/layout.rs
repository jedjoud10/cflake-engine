use std::mem::size_of;

use log_err::LogErrResult;
use vulkano::buffer::{DeviceLocalBuffer, BufferContents, CpuAccessibleBuffer};

use crate::{BufferMode, BufferUsage, Graphics};

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

// Convert the buffer type int to a Vulkano usage that we can add onto src_usage
pub(super) fn apply_buffer_type(layout: &mut BufferLayouts, _type: u32) {
    match _type {
        super::VERTEX => layout.src_usage = layout.src_usage.union(&vulkano::buffer::BufferUsage {
            vertex_buffer: true,
            ..Default::default()
        }),
        
        super::INDEX => layout.src_usage = layout.src_usage.union(&vulkano::buffer::BufferUsage {
            index_buffer: true,
            ..Default::default()
        }),

        super::STORAGE => layout.src_usage = layout.src_usage.union(&vulkano::buffer::BufferUsage {
            storage_buffer: true,
            ..Default::default()
        }),

        super::UNIFORM => layout.src_usage = layout.src_usage.union(&vulkano::buffer::BufferUsage {
            uniform_buffer: true,
            ..Default::default()
        }),

        super::INDIRECT => layout.src_usage = layout.src_usage.union(&vulkano::buffer::BufferUsage {
            indirect_buffer: true,
            ..Default::default()
        }),
        
        _ => ()
    }
}

// Create an empty buffer for a specific buffer type kind
pub(super) fn initialize_buffer<T: BufferContents>(graphics: &Graphics, length: usize, src_buffer_usage: vulkano::buffer::BufferUsage, src_host_cached: bool, kind: UniqueBufferLayoutKind) -> super::BufferKind<T> where [T]: BufferContents {
    match kind {
        UniqueBufferLayoutKind::DeviceLocal => {
            let buffer = DeviceLocalBuffer::<[T]>::array(
                graphics.memory_allocator(),
                length as u64,
                src_buffer_usage,
                [graphics.queue().queue_family_index()]
            ).expect("Could not create device local buffer");
            
            super::BufferKind::DeviceLocal(buffer)
        },
        UniqueBufferLayoutKind::CpuAccessible => unsafe {
            let buffer = CpuAccessibleBuffer::<[T]>::uninitialized_array(
                graphics.memory_allocator(),
                length as u64,
                src_buffer_usage,
                src_host_cached,
            ).expect("Could not create CPU accessible buffer");
            super::BufferKind::CpuAccessible(buffer)
        },
    }
}