use std::sync::{Arc, atomic::{Ordering, AtomicUsize}};

use utils::{ConcVec, AtomicBitSet};

use crate::Graphics;

// Create a BufferView for a specific staging buffer
pub(super) fn read_staging_buffer_view<'a>(
    graphics: &Graphics,
    buffer: &'a wgpu::Buffer,
    offset: u64,
    size: u64,
) -> wgpu::BufferView<'a> {
    // Map the staging buffer
    type MapResult = Result<(), wgpu::BufferAsyncError>;
    let (tx, rx) = std::sync::mpsc::channel::<MapResult>();

    // Map synchronously
    let slice = buffer.slice(offset..size);
    slice.map_async(wgpu::MapMode::Read, move |res| tx.send(res).unwrap());
    log::trace!("map buffer read: map called");
    graphics.device().poll(wgpu::Maintain::Wait);

    // Wait until the buffer is mapped, then read from the buffer
    if let Ok(Ok(_)) = rx.recv() {
        log::trace!("map buffer read");
        slice.get_mapped_range()
    } else {
        panic!("could not map buffer for reading")
    } 
}

// Create a StagingViewMut for a specific staging buffer
pub(super) fn write_staging_buffer_view_mut<'a>(
    graphics: &Graphics,
    buffer: &'a wgpu::Buffer,
    offset: u64,
    size: u64
) -> wgpu::BufferViewMut<'a> {
    // Map the staging buffer
    type MapResult = Result<(), wgpu::BufferAsyncError>;
    let (tx, rx) = std::sync::mpsc::channel::<MapResult>();

    // Map synchronously
    let slice = buffer.slice(offset..size);
    slice.map_async(wgpu::MapMode::Write, move |res| tx.send(res).unwrap());
    log::trace!("map buffer write: map called");
    graphics.device().poll(wgpu::Maintain::Wait);

    // Wait until the buffer is mapped, then read from the buffer
    if let Ok(Ok(_)) = rx.recv() {
        log::trace!("map buffer write");
        slice.get_mapped_range_mut()
    } else {
        panic!("could not map buffer for reading")
    } 
}

// Synchronously read the data from a staging buffer and write it to DST
pub(super) fn read_staging_buffer(
    graphics: &Graphics,
    buffer: &wgpu::Buffer,
    offset: u64,
    dst: &mut [u8],
) {
    let view = read_staging_buffer_view(graphics, buffer, offset, dst.len() as u64);
    dst.copy_from_slice(&*view);
}

// Synchronously write the data from a src buffer into a staging buffer
pub(super) fn write_staging_buffer(
    graphics: &Graphics,
    buffer: &wgpu::Buffer,
    offset: u64,
    data: &[u8],
) {
    graphics.queue().write_buffer(buffer, offset, data);
}

// Asynchronously read the data from a staging buffer
// This takes in self because we need to know the allocations count and shit
pub(super) fn async_read_staging_buffer(
    allocations: Arc<ConcVec<wgpu::Buffer>>,
    must_unmap: Arc<AtomicBitSet::<AtomicUsize>>,
    index: usize,
    offset: u64,
    size: u64,
    callback: impl FnOnce(&[u8]) + Send + 'static,
) {
    // Map asynchronously
    let slice = allocations[index].slice(offset..size);
    let allocations = allocations.clone();
    slice.map_async(wgpu::MapMode::Read, move |res| {
        log::trace!("map buffer read: map async resolved");
        res.unwrap();

        // Fetch staging buffer from self
        let buffer = &allocations[index];

        // Call the callback
        let slice = buffer.slice(offset..size);
        let bytes = &*slice.get_mapped_range();
        callback(bytes);

        // We must mark this buffer as a "must unmap" buffer
        must_unmap.set(index, Ordering::Relaxed);
    });

    drop(slice);
    log::trace!("map buffer read async: map async called");
}