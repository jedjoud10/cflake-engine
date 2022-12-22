use vulkan::{Recorder, vk, Allocation};
use crate::{Content, Graphics};
use super::{BufferLayouts};

// Allocate a new buffer with a specific size and layout
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
        write_to(block.mapped_slice_mut(), slice);
        copy_from_staging(block, size, recorder, queue, device, src_buffer);
    } else {
        // Write to the buffer memory by mapping it directly
        write_to(src_allocation.mapped_slice_mut().unwrap(), slice);
    }
    (src_buffer, src_allocation)
}

// Write to the given bytes slice
unsafe fn write_to<T: Content>(dst: &mut [u8], slice: &[T]) {
    let dst = bytemuck::cast_slice_mut::<u8, T>(dst);
    let len = slice.len();
    dst[..len].copy_from_slice(slice);
}

// Copy from the given staging block into the actual buffer
unsafe fn copy_from_staging<'a>(
    block: vulkan::StagingBlock,
    size: u64,
    recorder: &mut Recorder<'a>,
    queue: &'a vulkan::Queue,
    device: &'a vulkan::Device,
    src_buffer: vk::Buffer
) {
    let copy = *vk::BufferCopy::builder()
        .dst_offset(0)
        .src_offset(block.offset())
        .size(size);

    // Copy the contents from the staging buffer to the src buffer
    let mut old = std::mem::replace(
        recorder,
        queue.acquire(device),
    );

    // Record the cpy staging -> src buffer command
    recorder.cmd_full_barrier();
    old.cmd_copy_buffer(block.buffer(), src_buffer, &[copy]);
    recorder.cmd_full_barrier();
    
    // Submit the recorder
    queue.submit(old).wait();
}