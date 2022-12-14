use std::{
    marker::PhantomData,
    mem::{size_of, ManuallyDrop},
    ops::RangeBounds,
};

use crate::{
    BufferError, Graphics, InvalidModeError,
    InvalidUsageError, InitializationError,
};
use bytemuck::{Zeroable, Pod};
use vulkan::{Recorder, Allocation, vk};

// Some settings that tell us how exactly we should create the buffer
#[derive(Default, Clone, Copy, PartialEq, Eq, Debug)]
pub enum BufferMode {
    // Dynamic buffers are like static buffers, but they allow the user to mutate each element
    #[default]
    Dynamic,

    // Partial buffer have a fixed capacity, but a dynamic length
    Parital,

    // Resizable buffers can be resized to whatever length needed
    Resizable,
}

// How we shall access the buffer
// These buffer usages do not count the initial buffer creation phase
// Anything related to the device access is a hint since you can always access stuff
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub struct BufferUsage {
    // Specifies that the device can write to the buffer
    pub hint_device_write: bool,

    // Specifies that the device can read from the buffer
    pub hint_device_read: bool,

    // Specifies that the host can write to the buffer
    pub host_write: bool,

    // Specifies that the host can read from the buffer
    pub host_read: bool,
}

impl Default for BufferUsage {
    fn default() -> Self {
        Self {
            hint_device_write: false,
            hint_device_read: true,
            host_write: true,
            host_read: false,
        }
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
pub trait Content: Zeroable + Pod + Clone + Copy + Sync + Send + 'static {}
impl<T: Clone + Copy + Sync + Send + Zeroable + Pod + 'static> Content for T {}

// An abstraction layer over a valid OpenGL buffer
// This takes a valid OpenGL type and an element type, though the user won't be able make the buffer directly
// This also takes a constant that represents it's OpenGL target
pub struct Buffer<T: Content, const TYPE: u32> {
    buffer: vk::Buffer,
    allocation: ManuallyDrop<Allocation>,
    length: usize,
    capacity: usize,
    usage: BufferUsage,
    mode: BufferMode,
    graphics: Graphics,
    _phantom: PhantomData<T>,
}

// Internal bounds used by the buffer
pub(super) struct BufferBounds {
    offset: usize,
    size: usize,
}

impl<T: Content, const TYPE: u32> Buffer<T, TYPE> {
    // Create a buffer using a slice of elements
    // (will return none if we try to create a zero length Static, Dynamic, or Partial buffer)
    pub fn from_slice(
        graphics: &Graphics,
        slice: &[T],
        mode: BufferMode,
        usage: BufferUsage,
        recorder: &mut Recorder,
    ) -> Result<Self, BufferError> {
        // Cannot create a zero sized slice for non-resizable buffers
        if slice.is_empty() && !matches!(mode, BufferMode::Resizable) {
            return Err(BufferError::Initializion(InitializationError::NotResizable));
        }

        // Decompose graphics
        let device = graphics.device();
        let queues = graphics.queue();

        // Calculate the byte size of the buffer
        let size =
            u64::try_from(size_of::<T>() * slice.len()).unwrap();

        // Get location and staging buffer location
        let layout = super::find_optimal_layout(mode, usage, TYPE);

        // Create the actual buffer
        let (src_buffer, src_allocation) = unsafe {
            device.create_buffer(
                size,
                layout.src_buffer_usage_flags,
                layout.src_buffer_memory_location,
                graphics.queue(),
            )
        };

        /*

        // Optional init staging buffer
        let tmp_init_staging = layout
            .init_staging_buffer_memory_location
            .map(|memory| unsafe {
                let buffe = device.create_buffer(
                    size,
                    layout.init_staging_buffer_usage_flags.unwrap(),
                    graphics.queues(),
                );
                let memory =
                    device.create_buffer_memory(buffer, memory);
                (buffer, memory)
            });

        // Check if we need to make a staging buffer
        if let Some((buffer, allocation)) = tmp_init_staging {
            unsafe {
            }
        } else {
            // Write to the buffer memory by mapping it directly
            let dst = bytemuck::cast_slice_mut::<u8, T>(
                src_allocation.mapped_slice_mut().unwrap(),
            );
            let len = slice.len();
            dst[..len].copy_from_slice(slice);
        }

        // Create the struct and return it
        Some(Self {
            length: slice.len(),
            capacity: slice.len(),
            mode,
            usage,
            graphics: graphics.clone(),
            _phantom: PhantomData,
            buffer: src_buffer,
            allocation: ManuallyDrop::new(src_allocation),
        })
        */
        todo!()
    }

    // Create an empty buffer if we can (resizable)
    pub fn empty(
        graphics: &Graphics,
        mode: BufferMode,
        usage: BufferUsage,
        recorder: &mut Recorder,
    ) -> Result<Self, BufferError> {
        Self::from_slice(graphics, &[], mode, usage, recorder)
    }

    // Get the current length of the buffer
    pub fn len(&self) -> usize {
        self.length
    }

    // Check if the buffer is empty
    pub fn is_empty(&self) -> bool {
        self.length == 0
    }

    // Get the current capacity of the buffer
    pub fn capacity(&self) -> usize {
        self.capacity
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
    ) -> Result<BufferBounds, BufferError> {
        let start = match range.start_bound() {
            std::ops::Bound::Included(start) => *start,
            std::ops::Bound::Excluded(_) => panic!(),
            std::ops::Bound::Unbounded => 0,
        };

        let end = match range.end_bound() {
            std::ops::Bound::Included(end) => *end + 1,
            std::ops::Bound::Excluded(end) => *end,
            std::ops::Bound::Unbounded => self.length,
        };

        let valid_start_index = start < self.length;
        let valid_end_index = end <= self.length && end >= start;

        if !valid_start_index || !valid_end_index {
            return Err(BufferError::InvalidRangeSize(start, end, self.length));
        }

        if (end - start) == 0 {
            return Err(BufferError::InvalidRangeSize(start, end, self.length));
        }

        Ok(BufferBounds {
            offset: start,
            size: end - start,
        })
    }

    // Fills a range in the buffer with a constant value
    pub fn splat_range(
        &mut self,
        _val: T,
        range: impl RangeBounds<usize>,
        _recorder: &mut Recorder,
    ) -> Result<(), BufferError> {
        let Some(BufferBounds {
            offset: _, size: _ 
        }) = self.convert_range_bounds(range) else {
            return Ok(());
        };

        todo!()
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
        _recorder: &mut Recorder,
    ) -> Result<(), BufferError> {
        // Don't do anything
        if slice.is_empty() {
            return Ok(());
        }

        // Check if we are allowed to change the length of the buffer
        if !matches!(self.mode, BufferMode::Resizable)
            && !matches!(self.mode, BufferMode::Parital)
        {
            return Err(BufferError::InvalidMode(
                InvalidModeError::IllegalChangeLength,
            ));
        }

        // Check if we can write to the buffer
        if !self.usage.host_write {
            return Err(BufferError::InvalidUsage(
                InvalidUsageError::IllegalHostWrite,
            ));
        }

        // Check if we can read from the buffer
        if !self.usage.host_read {
            return Err(BufferError::InvalidUsage(
                InvalidUsageError::IllegalHostRead,
            ));
        }

        // Allocate the buffer for the first time
        if self.length == 0 && self.capacity == 0 {
            // TODO: Create the buffer
        } else if slice.len() + self.length > self.capacity {
            // create new buffer
            // cpy self -> new buffer
            // update self buffer
        } else {
            // TODO: write to current buffer
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
        let Some(BufferBounds {
            offset, size 
        }) = self.convert_range_bounds(range) else {
            return Ok(());
        };

        // Check if the given slice matches with the range
        if size != src.len() {
            return Err(BufferError::SliceLengthRangeMistmatch(
                src.len(),
                size,
            ));
        }

        // Check if we can write to the buffer
        if !self.usage.host_write {
            return Err(BufferError::InvalidUsage(
                InvalidUsageError::IllegalHostWrite,
            ));
        }

        // Get the mapped pointer and write to it the given slice (if possible)
        if let Some(mapped) = self.allocation.mapped_slice_mut() {
            let dst = bytemuck::cast_slice_mut::<u8, T>(mapped);
            dst[offset..size].copy_from_slice(src);
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
        let Some(BufferBounds {
            offset, size
        }) = self.convert_range_bounds(range) else {
            return Ok(());
        };

        // Check if the given slice matches with the range
        if size != dst.len() {
            return Err(BufferError::SliceLengthRangeMistmatch(
                dst.len(),
                size,
            ));
        }

        // Check if we can read from the buffer
        if !self.usage.host_read {
            return Err(BufferError::InvalidUsage(
                InvalidUsageError::IllegalHostRead,
            ));
        }

        // Get the mapped pointer and read from it (if possible)
        if let Some(mapped) = self.allocation.mapped_slice(){
            let src = bytemuck::cast_slice::<u8, T>(mapped);
            dst.copy_from_slice(&src[offset..size]);
        } else {
            // TODO: HANDLE READING FROM BUFFER THAT ISNT MAPPABLE
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
        let Some(BufferBounds {
            size, .. 
        }) = self.convert_range_bounds(range) else {
            return Ok(Vec::new());
        };

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
    pub fn clear(&mut self) -> Result<(), BufferError> {
        // TODO: write to current buffer
        self.length = 0;
        todo!()
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
        let Some(BufferBounds {
            offset, size
        }) = self.convert_range_bounds(src_range) else {
            return Ok(());
        };

        // Check if the given slice matches with the range
        if size != dst.len() {
            return Err(BufferError::SliceLengthRangeMistmatch(
                dst.len(),
                size,
            ));
        }

        // Check if we can read from the buffer
        if !self.usage.host_read {
            return Err(BufferError::InvalidUsage(
                InvalidUsageError::IllegalHostRead,
            ));
        }


        unsafe {
            recorder.cmd_copy_buffer(other.buffer, self.buffer, vec![
                
            ])
        }
    }

    // Copy the data from another buffer into this buffer
    pub fn copy_from<const OTHER: u32>(
        &mut self,
        other: &Buffer<T, OTHER>,
        recorder: &mut Recorder,
    ) -> Result<(), BufferError> {
        assert_eq!(
            self.len(),
            other.len(),
            "Cannot copy from buffer, length mismatch"
        );

        self.copy_range_from(.., other, 0, recorder);
    }

    // Fills the whole buffer with a constant value
    pub fn splat(
        &mut self,
        val: T,
        recorder: &mut Recorder,
    ) -> Result<(), BufferError> {
        self.splat_range(val, .., recorder)
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

impl<T: Content, const TYPE: u32> Drop for Buffer<T, TYPE> {
    fn drop(&mut self) {
        unsafe {
            let allocation = ManuallyDrop::take(&mut self.allocation);
            self.graphics
                .device()
                .destroy_buffer(self.buffer, allocation);
        }
    }
}