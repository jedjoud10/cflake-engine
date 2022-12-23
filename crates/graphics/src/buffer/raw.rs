use std::mem::size_of;

use vulkan::{Recorder, vk, Allocation, Submission};
use crate::{Content, Graphics};
use super::{BufferLayouts};

// Allocate a new buffer with a specific size and layout
// This will return the Vulkan buffer and memory allocation
pub(super) unsafe fn allocate_buffer<'a, T: Content>(
    graphics: &'a Graphics,
    size: u64,
    layout: BufferLayouts,
    slice: &[T],
    recorder: &mut Recorder<'a>
) -> (vk::Buffer, Allocation) {
    // Create the actual buffer
    let device = graphics.device();
    let queue = graphics.queue();
    let (src_buffer, mut src_allocation) = unsafe {
        device.create_buffer(
            size,
            layout.src_buffer_usage_flags,
            layout.src_buffer_memory_location,
            queue,
        )
    };
    
    // Get a free staging block with the given size
    let block =  layout.init_staging_buffer.then(|| unsafe {
        device.staging_pool().lock(device, queue, size)
    });

    // Check if we need to make a staging buffer
    if let Some(mut block) = block {
        // Write to the staging buffer and copy
        write_to(slice, block.mapped_slice_mut());
        copy_from_staging(
            &block,
            size,
            0,
            src_buffer,
            recorder,
            queue,
            device
        ).wait();
        device.staging_pool().unlock(device, block);
    } else {
        // Write to the buffer memory by mapping it directly
        write_to(slice, src_allocation.mapped_slice_mut().unwrap());
    }

    // Return the data
    (src_buffer, src_allocation)
}

// Write to the given bytes slice
pub(super) unsafe fn write_to<T: Content>(src: &[T], dst: &mut [u8]) {
    let dst = bytemuck::cast_slice_mut::<u8, T>(dst);
    let len = src.len();
    dst[..len].copy_from_slice(src);
}

// Read from the given bytes slice
pub(super) unsafe fn read_to<T: Content>(src: &[u8], dst: &mut [T]) {
    let src = bytemuck::cast_slice::<u8, T>(src);
    let len = dst.len();
    dst[..len].copy_from_slice(src);
}

// Perform a raw copy command from a staging buffer
pub(super)unsafe fn copy_from_staging<'a>(
    src_block: &vulkan::StagingBlock,
    size: u64,
    dst_offset: u64,
    dst_buffer: vk::Buffer,
    recorder: &mut Recorder<'a>,
    queue: &'a vulkan::Queue,
    device: &'a vulkan::Device,
) -> Submission<'a> {
    // Copy from the staging buffer
    let copy = *vk::BufferCopy::builder()
        .dst_offset(dst_offset)
        .src_offset(src_block.offset())
        .size(size);

    // Copy the contents from the staging buffer to the src buffer
    let mut old = std::mem::replace(
        recorder,
        queue.acquire(device),
    );

    // Record the cpy staging -> src buffer command
    recorder.cmd_full_barrier();
    old.cmd_copy_buffer(src_block.buffer(), dst_buffer, &[copy]);
    recorder.cmd_full_barrier();
    
    // Submit the recorder
    queue.submit(old)
}

// Perform a raw copy command into staging buffer
pub(super)unsafe fn copy_into_staging<'a>(
    dst_block: &vulkan::StagingBlock,
    size: u64,
    src_offset: u64,
    src_buffer: vk::Buffer,
    recorder: &mut Recorder<'a>,
    queue: &'a vulkan::Queue,
    device: &'a vulkan::Device,
) -> Submission<'a> {
    // Copy into the staging buffer
    let copy = *vk::BufferCopy::builder()
        .dst_offset(dst_block.offset())
        .src_offset(src_offset)
        .size(size);

    // Copy the contents from the staging buffer to the src buffer
    let mut old = std::mem::replace(
        recorder,
        queue.acquire(device),
    );

    // Record the cpy src buffer -> staging command
    recorder.cmd_full_barrier();
    old.cmd_copy_buffer(src_buffer, dst_block.buffer(), &[copy]);
    recorder.cmd_full_barrier();
    
    // Submit the recorder
    queue.submit(old)
}