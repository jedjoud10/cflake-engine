use std::{
    alloc::Layout,
    f32::consts::E,
    marker::PhantomData,
    mem::{size_of, ManuallyDrop, MaybeUninit},
    ops::RangeBounds,
};

use crate::{
    BufferError, BufferMode, BufferUsage, CopyError, ExtendError,
    Graphics, InitializationError, ReadError, WriteError,
};
use bytemuck::{Pod, Zeroable};
use vulkan::{vk, Allocation, Recorder};

// Bitmask from Vulkan BufferUsages
const VERTEX: u32 = vk::BufferUsageFlags::VERTEX_BUFFER.as_raw();
const INDEX: u32 = vk::BufferUsageFlags::INDEX_BUFFER.as_raw();
const STORAGE: u32 = vk::BufferUsageFlags::STORAGE_BUFFER.as_raw();
const UNIFORM: u32 = vk::BufferUsageFlags::UNIFORM_BUFFER.as_raw();
const INDIRECT: u32 = vk::BufferUsageFlags::INDIRECT_BUFFER.as_raw();

// Common buffer types
pub type VertexBuffer<T> = Buffer<T, VERTEX>;
pub type IndexBuffer<T> = Buffer<T, INDEX>;
pub type StorageBuffer<T> = Buffer<T, STORAGE>;
pub type UniformBuffer<T> = Buffer<T, UNIFORM>;
pub type IndirectBuffer<T> = Buffer<T, INDIRECT>;

// Plain old data type internally used by buffers and other types
pub trait Content:
    Zeroable + Pod + Clone + Copy + Sync + Send + 'static
{
}
impl<T: Clone + Copy + Sync + Send + Zeroable + Pod + 'static> Content
    for T
{
}

// An abstraction layer over a valid Vulkan buffer
// This also takes a constant that represents it's Vulkan target at compile time
// TODO: Handle async read writes and async command buf submissions
// TODO: Merge multiple async commands together? (like multiple copy or clear commands)
pub struct Buffer<T: Content, const TYPE: u32> {
    // Raw Vulkan
    buffer: vk::Buffer,
    allocation: ManuallyDrop<Allocation>,

    // Size fields
    length: usize,
    capacity: usize,

    // Legal Permissions
    usage: BufferUsage,
    mode: BufferMode,

    // Keep the graphics API alive
    graphics: Graphics,
    _phantom: PhantomData<T>,
}

impl<T: Content, const TYPE: u32> Drop for Buffer<T, TYPE> {
    fn drop(&mut self) {
        unsafe {
            let alloc = ManuallyDrop::take(&mut self.allocation);
            self.graphics.device().destroy_buffer(self.buffer, alloc);
        }
    }
}

// Implementation of util methods
impl<T: Content, const TYPE: u32> Buffer<T, TYPE> {
    // Get the inner raw Vulkan buffer
    pub fn raw(&self) -> Option<vk::Buffer> {
        (self.buffer != vk::Buffer::null()).then(|| self.buffer)
    }

    // Get the inner raw Vulkan Allocation immutably
    pub fn allocation(&self) -> &Allocation {
        &self.allocation
    }

    // Get the inner raw Vulkan Allocation mutably
    pub fn allocation_mut(&mut self) -> &mut Allocation {
        &mut self.allocation
    }

    // Get the current length of the buffer
    pub fn len(&self) -> usize {
        self.length.try_into().unwrap()
    }

    // Check if the buffer is empty
    pub fn is_empty(&self) -> bool {
        self.length == 0
    }

    // Get the current capacity of the buffer
    pub fn capacity(&self) -> usize {
        self.capacity.try_into().unwrap()
    }

    // Get the buffer mode
    pub fn mode(&self) -> BufferMode {
        self.mode
    }

    // Get the buffer's stride (length of each element)
    pub fn stride(&self) -> usize {
        size_of::<T>()
    }

    // Check if the buffer is HOST accessible (mappable)
    pub fn is_host_mapped(&self) -> bool {
        self.allocation().mapped_ptr().is_some()
    }
}

// Buffer initialization
impl<T: Content, const TYPE: u32> Buffer<T, TYPE> {
    // Try to create a buffer with the specified mode, usage, and slice data
    pub fn from_slice(
        graphics: &Graphics,
        slice: &[T],
        mode: BufferMode,
        usage: BufferUsage,
    ) -> Result<Self, BufferError> {
        // Cannot create a zero sized stride buffer
        assert!(
            size_of::<T>() > 0,
            "Buffers do not support zero-sized types"
        );

        // Cannot create a zero sized slice if we aren't resizable
        if slice.is_empty() && !matches!(mode, BufferMode::Resizable)
        {
            return Err(BufferError::Initialization(
                InitializationError::EmptySliceNotResizable,
            ));
        }

        // Get location and staging buffer location
        let (location, flags) =
            super::find_optimal_layout(usage, TYPE);

        // If the slice is empty (implying a resizable buffer), change it to contain one single null element
        // This is a little hack to allow us to have resizable buffers without dealing with null/invalid buffers
        let one = [T::zeroed()];
        let slice = if slice.is_empty() { &one } else { slice };

        // Allocate the buffer
        let (buffer, mut allocation) = unsafe {
            super::allocate_buffer::<T>(
                graphics,
                location,
                slice.len(),
                flags,
            )
        };

        // Fill up the buffer
        unsafe {
            super::fill_buffer(
                graphics,
                buffer,
                &mut allocation,
                slice,
            );
        }

        // Calculate the number of elements that can fit in this one allocation
        let stride = size_of::<T>() as u64;
        let capacity = (allocation.size() / stride) as usize;
        let allocation = ManuallyDrop::new(allocation);

        // Create the struct and return it
        Ok(Self {
            length: slice.len(),
            capacity,
            mode,
            usage,
            graphics: graphics.clone(),
            _phantom: PhantomData,
            buffer: buffer,
            allocation: allocation,
        })
    }

    // Create a buffer with a specific capacity and a length of 0
    pub fn with_capacity<'a>(
        graphics: &'a Graphics,
        capacity: usize,
        mode: BufferMode,
        usage: BufferUsage,
    ) -> Result<Self, BufferError> {
        let vec = vec![T::zeroed(); capacity];
        let mut buffer =
            Self::from_slice(graphics, &vec, mode, usage)?;
        buffer.length = 0;
        Ok(buffer)
    }
}

// Implementation of unsafe methods
impl<T: Content, const TYPE: u32> Buffer<T, TYPE> {
    // Read from "src" and write to buffer unsafely and instantly
    pub unsafe fn write_unchecked(
        &mut self,
        src: &[T],
        offset: usize,
    ) {
        if let Some(dst) = self.allocation_mut().mapped_slice_mut() {
            // Use the mapped pointer to write to the data
            let size = src.len() * size_of::<T>();
            let offset = offset * size_of::<T>();
            super::raw::write_to(src, &mut dst[offset..][..size]);
        } else {
            let device = self.graphics.device();
            let queue = self.graphics.queue();
            let mut recorder = queue.acquire(device);

            // Write to a staging buffer first
            let size = (src.len() * self.stride()) as u64;
            let offset = (offset * self.stride()) as u64;
            let mut block =
                device.staging_pool().lock(device, queue, size);
            let dst = block.mapped_slice_mut();
            super::raw::write_to(src, dst);

            // Copy from the staging buffer
            let copy = *vk::BufferCopy::builder()
                .dst_offset(offset)
                .src_offset(block.offset())
                .size(size);

            // Record the cpy staging -> src buffer command
            recorder.cmd_full_barrier();
            recorder.cmd_copy_buffer(
                block.buffer(),
                self.buffer,
                &[copy],
            );
            queue.submit(recorder).wait();

            device.staging_pool().unlock(device, block);
        }
    }

    // Read buffer and write to "dst" unsafely and instantly
    pub unsafe fn read_unchecked(
        &self,
        dst: &mut [T],
        offset: usize,
    ) {
        if let Some(src) = self.allocation().mapped_slice() {
            // Use the mapped pointer to read from the data
            let size = dst.len() * self.stride();
            let offset = offset * self.stride();
            super::raw::read_to(&src[offset..][..size], dst);
        } else {
            let device = self.graphics.device();
            let queue = self.graphics.queue();
            let mut recorder = queue.acquire(device);

            // Copy to a staging buffer first
            let size = (dst.len() * self.stride()) as u64;
            let offset = (offset * self.stride()) as u64;
            let block =
                device.staging_pool().lock(device, queue, size);

            // Copy into the staging buffer
            let copy = *vk::BufferCopy::builder()
                .dst_offset(block.offset())
                .src_offset(offset)
                .size(size);

            // Record the cpy src buffer -> staging command
            recorder.cmd_full_barrier();
            recorder.cmd_copy_buffer(
                self.buffer,
                block.buffer(),
                &[copy],
            );
            queue.submit(recorder).wait();

            // Read from the staging buffer and unlock it
            super::raw::read_to(block.mapped_slice(), dst);
            device.staging_pool().unlock(device, block);
        }
    }

    // Clear the buffer and reset it's length unsafely
    pub unsafe fn clear_unchecked(&mut self) {
        let device = self.graphics.device();
        let queue = self.graphics.queue();
        let mut recorder = queue.acquire(device);
        recorder.cmd_full_barrier();
        recorder.cmd_clear_buffer(self.buffer, 0, vk::WHOLE_SIZE);
        recorder.cmd_full_barrier();
        queue.submit(recorder).wait();
    }

    // Transmute the buffer into another type of buffer unsafely
    pub unsafe fn transmute<U: Content>(self) -> Buffer<U, TYPE> {
        assert_eq!(
            Layout::new::<T>(),
            Layout::new::<U>(),
            "Layout type mismatch, cannot transmute buffer"
        );

        let mut manually = ManuallyDrop::new(self);
        let allocation = std::mem::take(&mut manually.allocation);
        let graphics = manually.graphics.clone();

        Buffer::<U, TYPE> {
            buffer: manually.buffer,
            allocation,
            length: manually.length,
            capacity: manually.capacity,
            usage: manually.usage,
            mode: manually.mode,
            graphics,
            _phantom: PhantomData,
        }
    }

    // Copy the data from another buffer into this buffer unsafely and instantly
    pub unsafe fn copy_from_unchecked<const TYPE2: u32>(
        &mut self,
        src: &Buffer<T, TYPE2>,
        dst_offset: usize,
        src_offset: usize,
        length: usize,
    ) {
        let device = self.graphics.device();
        let queue = self.graphics.queue();
        let stride = self.stride();
        let mut recorder = queue.acquire(device);

        let dst_offset = (stride * dst_offset) as u64;
        let src_offset = (stride * src_offset) as u64;
        let size = (stride * length) as u64;

        // Create the copy info
        let copy = *vk::BufferCopy::builder()
            .dst_offset(dst_offset)
            .src_offset(src_offset)
            .size(size);

        // Record the cpy src -> self buffer command
        recorder.cmd_full_barrier();
        recorder.cmd_copy_buffer(src.buffer, self.buffer, &[copy]);
    }

    // Extend this buffer using the given slice unsafely and instantly
    pub unsafe fn extend_from_slice_unchecked(
        &mut self,
        slice: &[T],
    ) {
        if self.length + slice.len() > self.capacity {
            log::warn!("Reallocating buffer {:?}", self.buffer);

            // Calculate the new capacity
            let device = self.graphics.device();
            let queue = self.graphics.queue();
            let stride = self.stride();
            let mut recorder = queue.acquire(device);

            // Calculate the new capacity for the buffer
            let old_length = self.length;
            let new_capacity = self.capacity + slice.len();

            // Convert to bytes
            let old_length_bytes = (stride * old_length) as u64;

            // Allocate a new buffer with a bigger capacity
            let (location, flags) =
                super::find_optimal_layout(self.usage, TYPE);
            let (buffer, allocation) = unsafe {
                super::allocate_buffer::<T>(
                    &self.graphics,
                    location,
                    new_capacity,
                    flags,
                )
            };

            // Copy the old contents to the new buffer
            let copy = *vk::BufferCopy::builder()
                .dst_offset(0)
                .src_offset(0)
                .size(old_length_bytes);

            // Record the cpy src -> self buffer command
            recorder.cmd_full_barrier();
            recorder.cmd_copy_buffer(self.buffer, buffer, &[copy]);
            queue.submit(recorder).wait();

            // Overwrite the struct with the new buffer
            let old_buffer =
                std::mem::replace(&mut self.buffer, buffer);
            let mut old_allocation = std::mem::replace(
                &mut self.allocation,
                ManuallyDrop::new(allocation),
            );
            self.capacity = new_capacity;

            // Update the buffer with the new slice
            self.write_unchecked(slice, old_length);

            // Destroy the old buffer
            let alloc = ManuallyDrop::take(&mut old_allocation);
            self.graphics.device().destroy_buffer(old_buffer, alloc);
        } else {
            // Write to the buffer normally
            self.write_unchecked(slice, self.length);
        }

        self.length += slice.len();
    }
}

// Implementation of safe methods
impl<T: Content, const TYPE: u32> Buffer<T, TYPE> {
    // Read buffer and write to "dst" instantly
    pub fn write(
        &mut self,
        src: &[T],
        offset: usize,
    ) -> Result<(), BufferError> {
        if src.is_empty() {
            return Ok(());
        }

        if src.len() + offset > self.length {
            return Err(BufferError::WriteError(
                WriteError::InvalidLen(src.len(), offset, self.len()),
            ));
        }

        unsafe {
            self.write_unchecked(src, offset);
        }
        Ok(())
    }

    // Read from "src" and write to buffer instantly
    pub fn read<'a>(
        &'a self,
        dst: &mut [T],
        offset: usize,
    ) -> Result<(), BufferError> {
        if dst.is_empty() {
            return Ok(());
        }

        if dst.len() + offset > self.length {
            return Err(BufferError::ReadError(
                ReadError::InvalidLen(dst.len(), offset, self.len()),
            ));
        }

        unsafe {
            self.read_unchecked(dst, offset);
        }
        Ok(())
    }

    // Clear the buffer and reset it's length
    pub fn clear(&mut self) -> Result<(), BufferError> {
        if matches!(self.mode, BufferMode::Dynamic) {
            todo!()
        }

        unsafe {
            self.clear_unchecked();
        }
        Ok(())
    }

    // Copy the data from another buffer into this buffer instantly
    pub fn copy_from<const TYPE2: u32>(
        &mut self,
        src: &Buffer<T, TYPE2>,
        dst_offset: usize,
        src_offset: usize,
        length: usize,
    ) -> Result<(), BufferError> {
        if length == 0 {
            return Ok(());
        }

        if dst_offset + length > self.length {
            return Err(BufferError::CopyError(
                CopyError::InvalidDstOverflow(
                    length,
                    dst_offset,
                    self.len(),
                ),
            ));
        }

        if src_offset + length > src.length {
            return Err(BufferError::CopyError(
                CopyError::InvalidSrcOverflow(
                    length,
                    src_offset,
                    src.len(),
                ),
            ));
        }

        unsafe {
            self.copy_from_unchecked(
                src, dst_offset, src_offset, length,
            );
        }

        Ok(())
    }

    // Extend this buffer using the given slice instantly
    pub fn extend_from_slice(
        &mut self,
        slice: &[T],
    ) -> Result<(), BufferError> {
        if slice.is_empty() {
            return Ok(());
        }

        if matches!(self.mode, BufferMode::Dynamic) {
            return Err(BufferError::ExtendError(
                ExtendError::IllegalLengthModify,
            ));
        }

        if slice.len() + self.length > self.capacity
            && matches!(self.mode, BufferMode::Parital)
        {
            return Err(BufferError::ExtendError(
                ExtendError::IllegalReallocation,
            ));
        }

        unsafe {
            self.extend_from_slice_unchecked(slice);
        }
        Ok(())
    }

    // Try to view the buffer immutably (if it's mappable)
    pub fn as_slice(&self) -> Result<&[T], BufferError> {
        self.allocation()
            .mapped_slice()
            .map(|bytes| {
                &bytemuck::cast_slice::<u8, T>(bytes)[..self.length]
            })
            .ok_or(BufferError::NotMappable)
    }

    // Try to view the buffer mutably (if it's mappable)
    pub fn as_slice_mut(&mut self) -> Result<&mut [T], BufferError> {
        let length = self.length;
        self.allocation_mut()
            .mapped_slice_mut()
            .map(|bytes| {
                &mut bytemuck::cast_slice_mut::<u8, T>(bytes)
                    [..length]
            })
            .ok_or(BufferError::NotMappable)
    }
}
