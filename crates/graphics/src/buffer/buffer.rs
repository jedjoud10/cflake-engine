use std::{
    marker::PhantomData,
    mem::{size_of, ManuallyDrop, MaybeUninit},
    ops::RangeBounds,
};

use crate::{
    BufferError, ExtendFromIterError, Graphics, InitializationError,
    InvalidModeError, InvalidRangeSizeError, InvalidUsageError,
    WriteRangeError, CopyRangeFromError, ReadRangeError,
};
use bytemuck::{Pod, Zeroable};
use vulkan::{vk, Allocation, Recorder};

// Some settings that tell us how exactly we should create the buffer
#[derive(Default, Clone, Copy, PartialEq, Eq, Debug)]
pub enum BufferMode {
    // Dynamic buffers are like static buffers, but they allow the user to mutate each element
    Dynamic,

    // Partial buffer have a fixed capacity, but a dynamic length
    Parital,

    // Resizable buffers can be resized to whatever length needed
    #[default]
    Resizable,
}

// How we shall access the buffer
// These buffer usages do not count the initial buffer creation phase
// Anything related to the device access is a hint since you can always access stuff
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub struct BufferUsage {
    // Specifies what the device can do with the buffer
    pub device_write: bool,
    pub device_read: bool,

    // Specifies what the host can do do with the buffer
    pub host_write: bool,
    pub host_read: bool,
}

impl BufferUsage {    
    // Device local buffer usage. Not host visible
    pub fn device_local_usage() -> Self {
        Self {
            device_write: true,
            device_read: true,
            host_write: false,
            host_read: false,
        }
    }
    
    // Common buffer usage. Allows you to do anything
    pub fn common_device_usage() -> Self {
        Self {
            device_write: true,
            device_read: true,
            host_write: true,
            host_read: true,
        }
    }
    
    // Buffer usage to upload data to the GPU
    pub fn upload_to_device_usage() -> Self {
        Self {
            device_write: false,
            device_read: true,
            host_write: true,
            host_read: false,
        }
    }
    
    // Buffer usage to download data from the GPU
    pub fn download_from_device_usage() -> Self {
        Self {
            device_write: true,
            device_read: false,
            host_write: false,
            host_read: true,
        }
    }
}

impl Default for BufferUsage {
    fn default() -> Self {
        Self::common_device_usage()
    }
}

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
    length: u64,
    capacity: u64,

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

// Raw buffer allocation data that is used only internally
struct RawBufferAllocation {
    buffer: vk::Buffer,
    allocation: Allocation,
    capacity: u64,
}

impl<T: Content, const TYPE: u32> Buffer<T, TYPE> {
    // Allocate a raw Vulkan buffer and somehow write data to it    
    unsafe fn allocate_buffer<'a>(
        graphics: &'a Graphics,
        queue: &'a vulkan::Queue,
        device: &'a vulkan::Device,
        recorder: &mut Recorder<'a>,
        usage: BufferUsage,
        _type: u32,
        slice: &[T],
    ) -> RawBufferAllocation {
        // Calculate the byte size of the buffer
        let stride = size_of::<T>() as u64; 
        let size = stride * (slice.len() as u64);

        // Get location and staging buffer location
        let layout = super::find_optimal_layout(usage, _type);

        // Create the actual buffer
        let (src_buffer, mut src_allocation) = {
            device.create_buffer(
                size,
                layout.src_buffer_usage_flags,
                layout.src_buffer_memory_location,
                graphics.queue(),
            )
        };

        // Optional init staging buffer
        /*
        let tmp_init_staging =  layout.init_staging_buffer.then(|| {
            device.create_buffer(
                size,
                vk::BufferUsageFlags::TRANSFER_SRC,
                vulkan::MemoryLocation::CpuToGpu,
                queue,
        )});
        */

        let block =  layout.init_staging_buffer.then(|| {
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
            recorder.cmd_full_barrier();
            old.cmd_copy_buffer(block.buffer(), src_buffer, &[copy]);
            recorder.cmd_full_barrier();

            // Submit the recorder
            queue.submit(old).wait();
            //device.destroy_buffer(buffer, alloc);
            
        } else {
            // Write to the buffer memory by mapping it directly
            let dst = bytemuck::cast_slice_mut::<u8, T>(
                src_allocation.mapped_slice_mut().unwrap(),
            );
            let len = slice.len();
            dst[..len].copy_from_slice(slice);
        }

        RawBufferAllocation {
            buffer: src_buffer,
            allocation: src_allocation,
            capacity: size / stride,
        }
    }

    // Try to create a buffer with the specified mode, usage, and slice data
    pub fn from_slice<'a>(
        graphics: &'a Graphics,
        slice: &[T],
        mode: BufferMode,
        usage: BufferUsage,
        recorder: &mut Recorder<'a>,
    ) -> Result<Self, BufferError> {
        // Cannot create a zero sized slice for non-resizable buffers
        if slice.is_empty() && !matches!(mode, BufferMode::Resizable)
        {
            return Err(BufferError::Initialization(
                InitializationError::NotResizable,
            ));
        }

        // Cannot create a zero sized stride buffer
        if size_of::<T>() == 0 {
            return Err(BufferError::Initialization(
                InitializationError::ZeroSizedStride,
            ));
        }

        // Decompose graphics
        let device = graphics.device();
        let queue = graphics.queue();

        // If we are trying to create a zero sized resizable buffer, don't initialize the VK buffer
        if slice.is_empty() {
            return Ok(Self {
                buffer: vk::Buffer::null(),
                allocation: None,
                length: 0,
                capacity: 0,
                usage,
                mode,
                graphics: graphics.clone(),
                _phantom: PhantomData,
            })
        }

        // Create the buffer and write the data to it
        let RawBufferAllocation {
            buffer,
            allocation,
            capacity,
        } = unsafe {
            Self::allocate_buffer(
                graphics, queue, device, recorder,
                usage, TYPE, slice
            )
        };

        // Create the struct and return it
        Ok(Self {
            length: slice.len() as u64,
            capacity,
            mode,
            usage,
            graphics: graphics.clone(),
            _phantom: PhantomData,
            buffer,
            allocation: Some(allocation),
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
        Ok(buffer)
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
    // This will return None if the returning indices have a length of 0
    fn convert_range_bounds(
        &self,
        range: impl RangeBounds<usize>,
    ) -> Result<BufferBounds, InvalidRangeSizeError> {
        let length = self.length as usize;
        let start = match range.start_bound() {
            std::ops::Bound::Included(start) => *start,
            std::ops::Bound::Excluded(_) => panic!(),
            std::ops::Bound::Unbounded => 0,
        };

        let end = match range.end_bound() {
            std::ops::Bound::Included(end) => *end + 1,
            std::ops::Bound::Excluded(end) => *end,
            std::ops::Bound::Unbounded => length,
        };

        let valid_start_index = start < length;
        let valid_end_index = end <= length && end >= start;

        if !valid_start_index || !valid_end_index {
            return Err(InvalidRangeSizeError(
                start,
                end,
                length,
            ));
        }

        if (end - start) == 0 {
            return Err(InvalidRangeSizeError(
                start,
                end,
                length,
            ));
        }

        Ok(BufferBounds {
            offset: start,
            size: end - start,
        })
    }

    // Extent the current buffer using data from an iterator
    pub fn extend_from_iterator<I: Iterator<Item = T>>(
        &mut self,
        iterator: I,
        recorder: &mut Recorder,
    ) -> Result<(), BufferError> {
        let collected = iterator.collect::<Vec<_>>();
        self.extend_from_slice(&collected, recorder)
    }

    // Extend the current buffer using data from a new slice
    pub fn extend_from_slice(
        &mut self,
        slice: &[T],
        recorder: &mut Recorder,
    ) -> Result<(), BufferError> {
        // Don't do anything
        if slice.is_empty() {
            return Ok(());
        }

        // Check if we are allowed to change the length of the buffer
        if !matches!(self.mode, BufferMode::Resizable)
            && !matches!(self.mode, BufferMode::Parital)
        {
            return Err(BufferError::ExtendFromIter(
                ExtendFromIterError::InvalidMode(
                    InvalidModeError::IllegalChangeLength,
                ),
            ));
        }

        /*
        // Check if we can write to the buffer
        if !self.usage.host_write {
            return Err(BufferError::ExtendFromIter(
                ExtendFromIterError::InvalidUsage(
                    InvalidUsageError::IllegalHostWrite,
                ),
            ));
        }

        // Check if we can read from the buffer
        if !self.usage.host_read {
            return Err(BufferError::ExtendFromIter(
                ExtendFromIterError::InvalidUsage(
                    InvalidUsageError::IllegalHostWrite,
                ),
            ));
        }
        */

        // Allocate the buffer for the first time
        if self.len() == 0 && self.capacity() == 0 {
            log::warn!("Extending from slice: first time allocation");
            todo!();
            // TODO: Create the buffer
        } else if slice.len() + self.len() > self.capacity() {
            log::warn!("Extending from slice: creating larger buffer");
            todo!();
            // create new buffer
            // cpy self -> new buffer
            // update self buffer
        } else {
            log::warn!("Extending from slice: write to current buffer");
            // TODO: write to current buffer
            let old = self.len();
            self.length += slice.len() as u64;
            self.write_range(slice, old..self.len(), recorder).unwrap();
        }

        Ok(())
    }

    // Overwrite a region of the buffer using a slice and a range
    pub fn write_range(
        &mut self,
        src: &[T],
        range: impl RangeBounds<usize>,
        _recorder: &mut Recorder,
    ) -> Result<(), BufferError> {
        let BufferBounds { offset, size } =
            self.convert_range_bounds(range).map_err(|x| {
                BufferError::WriteRange(
                    WriteRangeError::InvalidRangeSize(x),
                )
            })?;


        // Check if the given slice matches with the range
        if size != src.len() {
            return Err(BufferError::WriteRange(WriteRangeError::SliceLengthMismatch()));
        }

        // Get the mapped pointer and write to it the given slice (if possible)
        if let Some(mapped) = self.allocation.as_mut().unwrap().mapped_slice_mut() {
            // Check if we can write to the buffer
            if !self.usage.host_write {
                return Err(BufferError::WriteRange(WriteRangeError::InvalidUsage(InvalidUsageError::IllegalHostWrite)));
            }

            let dst = &mut bytemuck::cast_slice_mut::<u8, T>(mapped);
            dst[offset..][..size].copy_from_slice(src);
        } else {
            // TODO: HANDLE WRITING TO BUFFER THAT ISNT MAPPABLE
            todo!()
        }
        Ok(())
    }

    // Read a region of the buffer into a mutable slice immediately
    pub fn read_range(
        &self,
        dst: &mut [T],
        range: impl RangeBounds<usize>,
        _recorder: &mut Recorder,
    ) -> Result<(), BufferError> {
        let BufferBounds { offset, size } =
            self.convert_range_bounds(range).map_err(|x| {
                BufferError::ReadRange(
                    ReadRangeError::InvalidRangeSize(x),
                )
            })?;


        // Check if the given slice matches with the range
        if size != dst.len() {
            return Err(BufferError::WriteRange(WriteRangeError::SliceLengthMismatch()));
        }

        // Get the mapped pointer and write to it the given slice (if possible)
        if let Some(mapped) = self.allocation.as_ref().unwrap().mapped_slice() {
            // Check if we can read from the buffer
            if !self.usage.host_read {
                return Err(BufferError::WriteRange(WriteRangeError::InvalidUsage(InvalidUsageError::IllegalHostRead)));
            }
            
            let src = &bytemuck::cast_slice::<u8, T>(mapped);
            dst.copy_from_slice(&src[offset..][..size]);
        } else {
            // TODO: HANDLE WRITING TO BUFFER THAT ISNT MAPPABLE
            todo!()
        }
        Ok(())
    }

    // Read a region of the buffer into a new vector
    pub fn read_range_as_vec(
        &self,
        range: impl RangeBounds<usize> + Copy,
        recorder: &mut Recorder,
    ) -> Result<Vec<T>, BufferError> {
        let BufferBounds { offset: _, size } =
        self.convert_range_bounds(range).map_err(|x| {
            BufferError::ReadRange(
                ReadRangeError::InvalidRangeSize(x),
            )
        })?;
        
        // Create a vec and read into it
        let mut vec = vec![T::zeroed(); size];
        self.read_range(&mut vec, range, recorder)?;
        Ok(vec)
    }

    // Read the whole buffer into a new vector
    pub fn read_to_vec(
        &self,
        recorder: &mut Recorder,
    ) -> Result<Vec<T>, BufferError> {
        self.read_range_as_vec(.., recorder)
    }

    // Clear the buffer contents, resetting the buffer's length down to zero
    pub fn clear(
        &mut self,
        recorder: &mut Recorder,
    ) -> Result<(), BufferError> {
        unsafe {
            let size = (self.len() * self.stride()) as u64;
            recorder.cmd_clear_buffer(self.buffer, 0, size);
        }
        self.length = 0;
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
        let BufferBounds {
            offset: src_offset, size
        } = self.convert_range_bounds(src_range).map_err(|x| {
            BufferError::CopyRangeFrom(
                CopyRangeFromError::InvalidDstRangeSize(x),
            )
        })?;

        // Check if the given range is valid for the other buffer
        if dst_offset + size < other.len() && size == other.len() {
            return Err(BufferError::CopyRangeFrom(CopyRangeFromError::InvalidSrcRangeSize(InvalidRangeSizeError(src_offset, src_offset+size, other.len()))));
        }

        // Check if we can write to the buffer
        if !self.usage.device_write {
            return Err(BufferError::CopyRangeFrom(
                CopyRangeFromError::InvalidSrcUsage(
                    InvalidUsageError::IllegalHostWrite,
                ),
            ));
        }

        // Check if we can read from the buffer
        if !other.usage.device_read {
            return Err(BufferError::CopyRangeFrom(
                CopyRangeFromError::InvalidDstUsage(
                    InvalidUsageError::IllegalHostWrite,
                ),
            ));
        }

        unsafe {
            let size = (size * self.stride()) as u64;
            let src_offset = (src_offset * self.stride()) as u64;
            let dst_offset = (dst_offset * self.stride()) as u64;
            let copy = *vk::BufferCopy::builder()
                .size(size)
                .src_offset(src_offset)
                .dst_offset(dst_offset);
            recorder.cmd_full_barrier();
            recorder.cmd_copy_buffer(other.buffer, self.buffer, &[copy]);
            recorder.cmd_full_barrier();
        }
        Ok(())
    }

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

    /*
    // Create a new ranged buffer reader that can read from the buffer
    pub fn as_view_ranged(&self, range: impl RangeBounds<usize>) -> Option<BufferView<T, TYPE>> {
        todo!()
    }

    // Create a new ranged buffer writer that can read/write from/to the buffer
    pub fn as_view_ranged_mut(
        &mut self,
        range: impl RangeBounds<usize>,
    ) -> Option<BufferViewMut<T, TYPE>> {
        todo!()
    }

    // Create a buffer reader that uses the whole buffer
    pub fn as_view(&self) -> Option<BufferView<T, TYPE>> {
        todo!()
    }

    // Create a buffer writer that uses the whole buffer
    pub fn as_mut_view(&mut self) -> Option<BufferViewMut<T, TYPE>> {
        todo!()
    }
    */
}