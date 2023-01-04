use std::mem::size_of;

use crate::{GpuPod, Graphics};
use vulkan::{vk, Allocation, MemoryLocation, Recorder};

// Allocate a new buffer with a specific size and layout
// This will return the Vulkan buffer and memory allocation
pub(super) unsafe fn allocate_buffer<'a, T: GpuPod>(
    graphics: &Graphics,
    location: MemoryLocation,
    capacity: usize,
    usage: vk::BufferUsageFlags,
) -> (vk::Buffer, Allocation) {
    let device = graphics.device();
    let queue = graphics.queue();
    let stride = size_of::<T>() as u64;
    let size = stride * (capacity as u64);

    // Create the actual buffer and it's memory allocation
    let (buffer, src_allocation) =
        unsafe { device.create_buffer(size, usage, location, queue) };

    (buffer, src_allocation)
}

// Fills a buffer with the specified data. Used for init
pub(super) unsafe fn fill_buffer<'a, T: GpuPod>(
    graphics: &Graphics,
    buffer: vk::Buffer,
    allocation: &mut Allocation,
    slice: &[T],
) {
    let device = graphics.device();
    let queue = graphics.queue();
    let mut recorder = queue.acquire(device);
    let device = graphics.device();
    let queue = graphics.queue();
    let stride = size_of::<T>() as u64;
    let size = stride * (slice.len() as u64);

    // Get a free staging block with the given size
    let block = matches!(MemoryLocation::GpuOnly, _location).then(
        || unsafe { device.staging_pool().lock(device, queue, size) },
    );

    // Check if we need to make a staging buffer
    if let Some(mut block) = block {
        // Write to the staging buffer and copy
        write_to(slice, block.mapped_slice_mut());

        // Copy from the staging buffer
        let copy = *vk::BufferCopy::builder()
            .dst_offset(0)
            .src_offset(block.offset())
            .size(size);

        // Record the cpy staging -> src buffer command
        recorder.cmd_full_pipeline_barrier();
        recorder.cmd_copy_buffer(block.buffer(), buffer, &[copy]);

        queue.chain(recorder);
        device.staging_pool().unlock(device, block);
    } else {
        // Write to the buffer memory by mapping it directly
        write_to(slice, allocation.mapped_slice_mut().unwrap());
    }

    // Return the data
}

// Write to the given bytes slice
pub(super) unsafe fn write_to<T: GpuPod>(src: &[T], dst: &mut [u8]) {
    assert_eq!(src.len(), dst.len() / size_of::<T>());
    let dst = bytemuck::cast_slice_mut::<u8, T>(dst);
    dst.copy_from_slice(src);
}

// Read from the given bytes slice
pub(super) unsafe fn read_to<T: GpuPod>(src: &[u8], dst: &mut [T]) {
    assert_eq!(dst.len(), src.len() / size_of::<T>());
    let src = bytemuck::cast_slice::<u8, T>(src);
    dst.copy_from_slice(src);
}

// Perform a raw copy command from a staging buffer
pub(super) unsafe fn copy_from_staging<'a>(
    src_block: &vulkan::StagingBlock,
    size: u64,
    dst_offset: u64,
    dst_buffer: vk::Buffer,
    recorder: &mut Recorder<'a>,
) {
    // Copy from the staging buffer
    let copy = *vk::BufferCopy::builder()
        .dst_offset(dst_offset)
        .src_offset(src_block.offset())
        .size(size);

    // Record the cpy staging -> src buffer command
    recorder.cmd_full_pipeline_barrier();
    recorder.cmd_copy_buffer(src_block.buffer(), dst_buffer, &[copy]);
}

// Perform a raw copy command into staging buffer
pub(super) unsafe fn copy_into_staging<'a>(
    dst_block: &vulkan::StagingBlock,
    size: u64,
    src_offset: u64,
    src_buffer: vk::Buffer,
    recorder: &mut Recorder<'a>,
) {
    // Copy into the staging buffer
    let copy = *vk::BufferCopy::builder()
        .dst_offset(dst_block.offset())
        .src_offset(src_offset)
        .size(size);

    // Record the cpy src buffer -> staging command
    recorder.cmd_full_pipeline_barrier();
    recorder.cmd_copy_buffer(src_buffer, dst_block.buffer(), &[copy]);
}
