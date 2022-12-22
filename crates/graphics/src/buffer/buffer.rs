use std::{
    marker::PhantomData,
    mem::{size_of, ManuallyDrop, MaybeUninit},
    ops::RangeBounds,
};

use crate::{
    BufferError, Graphics, InitializationError, InvalidModeError,
    InvalidRangeSizeError, InvalidUsageError, BufferUsage, BufferMode,
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
pub struct Buffer<T: Content, const TYPE: u32> {
    // Raw Vulkan
    buffer: vk::Buffer,
    allocation: Option<Allocation>,
    
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
            if let Some(allocation) = self.allocation.take() {
                self.graphics
                   .device()
                   .destroy_buffer(self.buffer, allocation);
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

        let valid_start_index = start < length;
        let valid_end_index = end <= length && end >= start;
        let valid_length = (end - start) > 0;

        if !valid_start_index || !valid_end_index || !valid_length {
            return Err(BufferError::InvalidRange(InvalidRangeSizeError(
                start,
                end,
                length,
            )));
        }

        Ok(BufferBounds {
            offset: start,
            size: end - start,
        })
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
        // Cannot create a zero sized slice
        if slice.is_empty() {
            return Err(BufferError::Initialization(
                InitializationError::ZeroSizedSlice,
            ));
        }

        // Cannot create a zero sized stride buffer
        if size_of::<T>() == 0 {
            return Err(BufferError::Initialization(
                InitializationError::ZeroSizedStride,
            ));
        }

        let device = graphics.device();
        let queue = graphics.queue();

        // Calculate the byte size of the buffer
        let stride = size_of::<T>() as u64; 
        let size = stride * (slice.len() as u64);

        // Get location and staging buffer location
        let layout = super::find_optimal_layout(usage, TYPE);

        // Create the actual buffer
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
            // Write to the staging buffer memory by mapping it directly
            let dst = bytemuck::cast_slice_mut::<u8, T>(
                block.mapped_slice_mut(),
            );
            let len = slice.len();
            dst[..len].copy_from_slice(slice);

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
            unsafe {
                recorder.cmd_full_barrier();
                old.cmd_copy_buffer(block.buffer(), src_buffer, &[copy]);
                recorder.cmd_full_barrier();
            }

            // Submit the recorder
            queue.submit(old).wait();            
        } else {
            // Write to the buffer memory by mapping it directly
            let dst = bytemuck::cast_slice_mut::<u8, T>(
                src_allocation.mapped_slice_mut().unwrap(),
            );
            let len = slice.len();
            dst[..len].copy_from_slice(slice);
        }

        // Create the struct and return it
        Ok(Self {
            length: slice.len(),
            capacity: (src_allocation.size() / stride) as usize,
            mode,
            usage,
            graphics: graphics.clone(),
            _phantom: PhantomData,
            buffer: src_buffer,
            allocation: Some(src_allocation),
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

    /*
    // Create a buffer with a specific capacity
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

        if matches!(BufferMode::Dynamic, mode) {
            return Err(BufferError::Initialization(InitializationError::ZeroSizedSlice));
        }

        Ok(buffer)
    }
    */
}

// Implementation of safe methods
impl<T: Content, const TYPE: u32> Buffer<T, TYPE> {
    // Overwrite a region of the buffer using a slice and a range
    pub fn write_range(
        &mut self,
        src: &[T],
        range: impl RangeBounds<usize>,
        recorder: &mut Recorder,
    ) -> Result<(), BufferError> {
        let BufferBounds {
            offset,
            size,
        } = self.convert(range)?;
        Ok(())
    }

    // Read a region of the buffer into a mutable slice immediately
    pub fn read_range(
        &self,
        dst: &mut [T],
        range: impl RangeBounds<usize>,
        _recorder: &mut Recorder,
    ) -> Result<(), BufferError> {
        let BufferBounds {
            offset,
            size,
        } = self.convert(range)?;
        Ok(())
    }

    // Clear the buffer contents, resetting the buffer's length down to zero
    pub fn clear(
        &mut self,
        recorder: &mut Recorder,
    ) -> Result<(), BufferError> {
        Ok(())
    }

    // Copy the data from another buffer's range into this buffer's range
    // dst_offset refers to the offset inside Self
    // src_range refers to the range of 'other'
    pub fn copy_range_from<const OTHER_TYPE: u32>(
        &mut self,
        src_range: impl RangeBounds<usize>,
        other: &Buffer<T, OTHER_TYPE>,
        dst_offset: usize,
        recorder: &mut Recorder,
    ) -> Result<(), BufferError> {
        Ok(())
    }
}

// Default methods (.. as range)
impl<T: Content, const TYPE: u32> Buffer<T, TYPE> {
    // Copy the data from another buffer into this buffer
    pub fn copy_from<const OTHER: u32>(
        &mut self,
        other: &Buffer<T, OTHER>,
        recorder: &mut Recorder,
    ) -> Result<(), BufferError> {
        self.copy_range_from(.., other, 0, recorder)
    }

    // Overwrite the whole buffer using a slice
    pub fn write(
        &mut self,
        slice: &[T],
        recorder: &mut Recorder,
    ) -> Result<(), BufferError> {
        self.write_range(slice, .., recorder)
    }

    // Read the whole buffer into a mutable slice
    pub fn read(
        &self,
        slice: &mut [T],
        recorder: &mut Recorder,
    ) -> Result<(), BufferError> {
        self.read_range(slice, .., recorder)
    }
}