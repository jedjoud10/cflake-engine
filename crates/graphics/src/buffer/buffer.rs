use std::{
    marker::PhantomData,
    mem::{size_of, ManuallyDrop, MaybeUninit},
    ops::RangeBounds, alloc::Layout, f32::consts::E,
};

use crate::{
    BufferError, Graphics, InitializationError, InvalidModeError,
    InvalidRangeSizeError, InvalidUsageError, BufferUsage, BufferMode,
};
use super::BufferLayouts;
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
pub struct Buffer<T: Content, const TYPE: u32> {
    // Raw Vulkan
    buffer: vk::Buffer,
    allocation: ManuallyDrop<MaybeUninit<Allocation>>,
    
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
            if self.buffer == vk::Buffer::null() {
                let alloc = ManuallyDrop::take(&mut self.allocation);
                let alloc = alloc.assume_init();
                self.graphics
                   .device()
                   .destroy_buffer(self.buffer, alloc);
            }       
        }
    }
}

// Internal bounds used by the buffer
pub(super) struct BufferBounds {
    offset: usize,
    size: usize,
}

// Implementation of util methods
impl<T: Content, const TYPE: u32> Buffer<T, TYPE> {
    // Get the inner raw Vulkan buffer
    pub fn raw(&self) -> Option<vk::Buffer> {
        (self.buffer != vk::Buffer::null()).then(|| self.buffer)
    }

    // Get the inner raw Vulkan Allocation immutably
    pub fn allocation(&self) -> Option<&Allocation> {
        (self.buffer != vk::Buffer::null()).then(|| unsafe {
            self.allocation.assume_init_ref()
        })
    }

    // Get the inner raw Vulkan Allocation mutably
    pub fn allocation_mut(&mut self) -> Option<&mut Allocation> {
        (self.buffer != vk::Buffer::null()).then(|| unsafe {
            self.allocation.assume_init_mut()
        })
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

    // Convert a range bounds type into the range indices
    // This will take the size in 
    fn convert(
        &self,
        range: impl RangeBounds<usize>,
    ) -> Result<BufferBounds, BufferError> {
        let length = self.length as usize;

        // Convert the end range index to the element index
        let start = match range.start_bound() {
            std::ops::Bound::Included(start) => *start,
            std::ops::Bound::Excluded(_) => panic!(),
            std::ops::Bound::Unbounded => 0,
        };

        // Conver the end range index to the element index
        let end = match range.end_bound() {
            std::ops::Bound::Included(end) => *end + 1,
            std::ops::Bound::Excluded(end) => *end,
            std::ops::Bound::Unbounded => length,
        };

        // Multiple checks to make sure the range is valid
        let valid_start_index = start < length;
        let valid_end_index = end <= length && end >= start;
        let valid_length = (end - start) > 0;

        // Handle errors
        if valid_start_index && valid_end_index && valid_length {
            Ok(BufferBounds {
                offset: start,
                size: end - start,
            })
        } else {
            Err(BufferError::InvalidRange(InvalidRangeSizeError(
                start,
                end,
                length,
            )))
        }
        
    }
}

// Buffer initialization
impl<T: Content, const TYPE: u32> Buffer<T, TYPE> {
    // Try to create a buffer with the specified mode, usage, and slice data
    pub fn from_slice<'a>(
        graphics: &'a Graphics,
        slice: &[T],
        mode: BufferMode,
        usage: BufferUsage,
        recorder: &mut Recorder<'a>,
    ) -> Result<Self, BufferError> {
        // Cannot create a zero sized stride buffer
        assert!(size_of::<T>() > 0, "Buffers do not support zero-sized types");        

        // Cannot create a zero sized slice if we aren't resizable
        if slice.is_empty() && !matches!(mode, BufferMode::Resizable) {
            return Err(BufferError::Initialization(
                InitializationError::EmptySliceNotResizable,
            ));
        }

        // Calculate the byte size of the buffer
        let stride = size_of::<T>() as u64; 
        let size = stride * (slice.len() as u64);

        // Get location and staging buffer location
        let layout = super::find_optimal_layout(usage, TYPE);

        // Allocate an actual buffer if we have valid data
        let (buffer, allocation, capacity) = if !slice.is_empty() {
            let (buffer, allocation) = unsafe { super::allocate_buffer(
                graphics,
                size,
                layout,
                slice,
                recorder
            ) };

            // Calculate the number of elements that can fit in this one allocation
            let capacity = (allocation.size() / stride) as usize;
            let allocation = ManuallyDrop::new(MaybeUninit::new(allocation));
            (buffer, allocation, capacity)
        } else {
            // Don't allocate a buffer nor allocate it's memory
            (vk::Buffer::null(), ManuallyDrop::new(MaybeUninit::uninit()), 0)
        };

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

    // Create an empty buffer if we can (resizable)
    pub fn empty<'a>(
        graphics: &'a Graphics,
        mode: BufferMode,
        usage: BufferUsage,
        recorder: &mut Recorder<'a>,
    ) -> Result<Self, BufferError> {
        Self::from_slice(graphics, &[], mode, usage, recorder)
    }

    // Create a buffer with a specific capacity and a length of 0
    pub fn with_capacity<'a>(
        graphics: &'a Graphics,
        capacity: usize,
        mode: BufferMode,
        usage: BufferUsage,
        recorder: &mut Recorder<'a>,
    ) -> Result<Self, BufferError> {
        let vec = vec![T::zeroed(); capacity];
        let mut buffer = Self::from_slice(graphics, &vec, mode, usage, recorder)?;
        buffer.length = 0;
        Ok(buffer)
    }
}

// Implementation of unsafe methods
impl<T: Content, const TYPE: u32> Buffer<T, TYPE> {
    // Read from "src" and write to buffer unsafely and instantly
    pub unsafe fn write_unchecked<'a>(
        &'a mut self,
        src: &[T],
        offset: usize,
        recorder: &mut Recorder<'a>,
    ) {
        if self.usage.host_write {
            // Use the mapped point to write to the data
            let dst = self.allocation_mut().unwrap_unchecked().mapped_slice_mut().unwrap_unchecked();
            super::raw::write_to(src, dst);
        } else {
            let device = self.graphics.device();
            let queue = self.graphics.queue();
            
            // Write to a staging buffer first
            let size = ((self.len() - offset) * self.stride()) as u64; 
            let offset = (offset * self.stride()) as u64;
            let mut block = device.staging_pool().lock(device, queue, size);
            super::raw::write_to(src, block.mapped_slice_mut());

            // Copy the data from the staging buffer
            super::raw::copy_from_staging(
                &block,
                size,
                offset,
                self.buffer,
                recorder,
                queue,
                device
            ).wait();
            device.staging_pool().unlock(device, block);
        }
    }
    
    // Read buffer and write to "dst" unsafely and instantly
    pub unsafe fn read_unchecked<'a>(
        &'a self,
        dst: &mut [T],
        offset: usize,
        recorder: &mut Recorder<'a>,
    ) {        
        if self.usage.host_read {
            // Use the mapped pointer to read from the data
            let src = self.allocation().unwrap_unchecked().mapped_slice().unwrap_unchecked();
            super::raw::read_to(src, dst);
        } else {
            let device = self.graphics.device();
            let queue = self.graphics.queue();
            
            // Copy to a staging buffer first
            let size = ((self.len() - offset) * self.stride()) as u64; 
            let offset = (offset * self.stride()) as u64;
            let block = device.staging_pool().lock(device, queue, size);

            // Copy the data into the staging buffer
            super::raw::copy_into_staging(
                &block,
                size,
                offset,
                self.buffer,
                recorder,
                queue,
                device
            ).wait();

            // Read from the staging buffer and unlock it
            super::raw::read_to(block.mapped_slice(), dst);
            device.staging_pool().unlock(device, block);
        }
    }

    // Clear the buffer and reset it's length unsafely
    pub unsafe fn clear_unchecked(
        &mut self,
        recorder: &mut Recorder
    ) {
        recorder.cmd_full_barrier();
        recorder.cmd_clear_buffer(self.buffer, 0, vk::WHOLE_SIZE);
        recorder.cmd_full_barrier();
    }

    // Transmute the buffer into another type of buffer unsafely
    pub unsafe fn transmute<U: Content>(self) -> Buffer<U, TYPE> {
        assert_eq!(
            Layout::new::<T>(),
            Layout::new::<U>(),
            "Layout type mismatch, cannot transmute buffer"
        );

        let mut manually = ManuallyDrop::new(self);
        let allocation = std::mem::replace(
            &mut manually.allocation,
            ManuallyDrop::new(MaybeUninit::uninit())
        );
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

    // Copy the data from another buffer's range into this buffer's range unsafely
    // Copy the data from another buffer into this buffer unsafely
}

// Implementation of safe methods
impl<T: Content, const TYPE: u32> Buffer<T, TYPE> {
    // Read buffer and write to "dst" instantly
    pub fn write<'a>(
        &'a mut self,
        src: &[T],
        offset: usize,
        recorder: &mut Recorder<'a>,
    ) {
        
        self.write_unchecked(src, offset, recorder);
    }

    // Read from "src" and write to buffer instantly
    pub fn read<'a>(
        &'a self,
        dst: &mut [T],
        offset: usize,
        recorder: &mut Recorder<'a>,
    ) {}

    // Clear the buffer and reset it's length
    // Copy the data from another buffer's range into this buffer's range
    // Copy the data from another buffer into this buffer
}