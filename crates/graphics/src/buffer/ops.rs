use std::{
    alloc::Layout,
    any::type_name,
    f32::consts::E,
    marker::PhantomData,
    mem::{size_of, ManuallyDrop},
    num::NonZeroU64,
    ops::{Bound, Range, RangeBounds},
};

use bytemuck::{Pod, Zeroable};
use wgpu::{util::DeviceExt, CommandEncoder, Maintain};


// FIXME: Is this really worth it? All of this just so we can remove the "range" parameter?
// FIXME: If we do remove this it's a big bonus since we stay consistent with how the new texture API fetching will work
/*
// Operations that we can do on specfic buffer slices/buffers
pub trait RangedBufferOps<T: GpuPod, const TYPE: u32> {
// Read from "src" and write to the buffer instantly
    // This is a "fire and forget" command that does not stall the CPU
    // The user can do multiple write calls and expect them to be batched together
    pub fn write(&mut self, src: &[T], offset: usize) -> Result<(), BufferWriteError> {
        // Nothing to read from
        if src.is_empty() {
            return Ok(());
        }

        // Make sure we can write to the buffer
        if !self.usage.contains(BufferUsage::WRITE) {
            return Err(BufferWriteError::NonWritable);
        }

        // Make sure the "offset" doesn't cause writes outside the buffer
        if src.len() + offset > self.length {
            return Err(BufferWriteError::InvalidLen(src.len(), offset, self.len()));
        }

        // Use the staging pool for data writes
        let staging = self.graphics.staging_pool();
        staging.write_buffer(
            &self.graphics,
            &self.buffer,
            (offset * self.stride()) as u64,
            (src.len() * self.stride()) as u64,
            bytemuck::cast_slice(src),
        );

        Ok(())
    }

    // Read buffer and write to "dst" instantly
    // Will stall the CPU, since this is waiting for GPU data
    fn read<'a>(&'a self, dst: &mut [T], offset: usize) -> Result<(), BufferReadError> {
        // Nothing to write to
        if dst.is_empty() {
            return Ok(());
        }

        // Make sure we can read from the buffer
        if !self.usage.contains(BufferUsage::READ) {
            return Err(BufferReadError::NonReadable);
        }

        // Make sure the "offset" doesn't cause reads outside the buffer
        if dst.len() + offset > self.length {
            return Err(BufferReadError::InvalidLen(dst.len(), offset, self.len()));
        }

        // Use the staging pool for data reads
        let staging = self.graphics.staging_pool();
        staging.read_buffer(
            &self.graphics,
            &self.buffer,
            (offset * self.stride()) as u64,
            (dst.len() * self.stride()) as u64,
            bytemuck::cast_slice_mut(dst),
        );

        Ok(())
    }

    // Read the buffer ASYNCHRONOUSLY without stalling
    // The read will be completed at the end of the frame when WGPU polls the device
    // Only works on dynamic buffers since they have a constant length throuhgout their lifetime
    fn async_read<'a>(&'a self, callback: impl FnOnce(&[T]) + Sync + Send + 'static) -> Result<(), BufferAsyncReadError> {
        let (start, end) = self
            .convert_bounds_to_indices(range)
            .ok_or(BufferAsyncReadError::InvalidRange(self.length))?;
        let len = end - start;
        let offset = start;

        // Make sure we can read from the buffer
        if !self.usage.contains(BufferUsage::READ) {
            return Err(BufferAsyncReadError::NonReadable);
        }

        // Use the staging pool for data reads
        let staging = self.graphics.staging_pool();
        staging.map_buffer_read_async(
            &self.graphics,
            &self.buffer,
            (offset * self.stride()) as u64,
            (len * self.stride()) as u64,
            |raw| callback(bytemuck::cast_slice(raw)),
        );

        Ok(())
    }

    // Fill a buffer region with a repeating value specified by "val"
    // This is a "fire and forget" command that does not stall the CPU
    // The user can do multiple splat calls and expect them to be batched together
    fn splat(&mut self, val: T) -> Result<(), BufferSplatError> {
        let (start, end) = self
            .convert_bounds_to_indices(range)
            .ok_or(BufferSplatError::InvalidRange(self.length))?;
        let len = end - start;

        let src = vec![val; len];
        self.write(&src, start).map_err(|err| match err {
            BufferWriteError::InvalidLen(_, _, _) => panic!(),
            BufferWriteError::NonWritable => BufferSplatError::NonWritable,
        })?;

        Ok(())
    }

    // Copy the data from another buffer into this buffer instantly
    // This is a "fire and forget" command that does not stall the CPU
    // The user can do multiple copy_from calls and expect them to be batched together
    fn copy_from<const TYPE2: u32>(
        &mut self,
        src: &Buffer<T, TYPE2>,
        offset: usize,
    ) -> Result<(), BufferCopyError> {
        if length == 0 {
            return Ok(());
        }

        if !self.usage.contains(BufferUsage::COPY_DST) {
            return Err(BufferCopyError::NonCopyDst);
        }

        if !src.usage.contains(BufferUsage::COPY_SRC) {
            return Err(BufferCopyError::NonCopySrc);
        }

        if dst_offset + length > self.length {
            return Err(BufferCopyError::InvalidDstOverflow(
                length,
                dst_offset,
                self.len(),
            ));
        }

        if src_offset + length > src.length {
            return Err(BufferCopyError::InvalidSrcOverflow(
                length,
                src_offset,
                src.len(),
            ));
        }

        // Calculate byte wise offsets and sizes
        let source_offset = (src_offset * self.stride()) as u64;
        let destination_offset = (dst_offset * self.stride()) as u64;
        let copy_size = (length * self.stride()) as u64;

        let mut encoder = self.graphics.acquire();
        encoder.copy_buffer_to_buffer(
            src.raw(),
            source_offset,
            self.raw(),
            destination_offset,
            copy_size,
        );
        self.graphics.reuse([encoder]);
        Ok(())
    }

    // Try to view the buffer immutably immediately
    // Will stall the CPU, since this is synchronous
    fn as_view(&self) -> Result<BufferView<T, TYPE>, BufferNotMappableError> {
        if !self.usage.contains(BufferUsage::READ) {
            return Err(BufferNotMappableError::AsView);
        }

        // Size and offset of the slice
        let (start, end) = self
            .convert_bounds_to_indices(bounds)
            .ok_or(BufferNotMappableError::InvalidRange(self.length))?;
        let size = (end - start) * self.stride();
        let offset = start * self.stride();

        // Get the staging pool for download
        let staging = self.graphics.staging_pool();
        let data = staging
            .map_buffer_read(&self.graphics, &self.buffer, offset as u64, size as u64);

        Ok(BufferView {
            buffer: &self,
            data,
        })
    }

    // Try to view the buffer mutably (for writing AND reading) immediately
    // Will stall the CPU, since this is synchronous
    // If the BufferUsage is Write only, then reading from BufferViewMut might be slow / might not return buffer contents
    // If the user tries to read an immutable slice from the BufferView then the program will panic
    fn as_view_mut(&mut self) -> Result<BufferViewMut<T, TYPE>, BufferNotMappableError> {
        if !self.usage.contains(BufferUsage::WRITE) {
            return Err(BufferNotMappableError::AsViewMut);
        }

        // Size and offset of the slice
        let (start, end) = self
            .convert_bounds_to_indices(bounds)
            .ok_or(BufferNotMappableError::InvalidRange(self.length))?;
        let size = (end - start) * self.stride();
        let offset = start * self.stride();

        // Check if we can read the buffer
        let read = self.usage.contains(BufferUsage::READ);

        if !read {
            // Write only buffer view, uses QueueWriteBufferView
            let staging = self.graphics.staging_pool();
            let data = staging
                .map_buffer_write(&self.graphics, &self.buffer, offset as u64, size as u64)
                .unwrap();
            Ok(BufferViewMut::Mapped {
                buffer: PhantomData,
                data,
            })
        } else {
            // Read and write, clone first, then write
            // Create a temporary vector that will store the contents of the buffer
            let mut vector = vec![T::zeroed(); end - start];
            self.read(&mut vector, start).unwrap();

            Ok(BufferViewMut::Cloned {
                buffer: self,
                data: vector,
            })
        }
    }
}
*/
