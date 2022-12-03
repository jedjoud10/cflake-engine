use std::{
    marker::PhantomData,
    mem::{size_of, ManuallyDrop},
    ops::RangeBounds,
};

use crate::{
    BufferError, Graphics, InvalidModeError,
    InvalidUsageError,
};
use bytemuck::Zeroable;
use vulkano::{buffer::{DeviceLocalBuffer, BufferContents, CpuAccessibleBuffer}, command_buffer::{AutoCommandBufferBuilder, PrimaryCommandBufferAbstract, PrimaryAutoCommandBuffer}};

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

// How we shall access the buffer on the CPU and GPU
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub struct BufferUsage {
    // Only optimization hints, we don't *actually* need them
    pub hint_device_write: bool,
    pub hint_device_read: bool,

    // **REQUIRED** if we wish to use the buffer on the CPU side 
    pub permission_host_write: bool,
    pub permission_host_read: bool,
}

impl Default for BufferUsage {
    fn default() -> Self {
        Self {
            hint_device_write: false,
            hint_device_read: true,
            permission_host_write: false,
            permission_host_read: false,
        }
    }
}

// Bitmask from WGPU BufferUsages
const VERTEX: u32 = 0;
const INDEX: u32 = 1;
const STORAGE: u32 = 2;
const UNIFORM: u32 = 3;
const INDIRECT: u32 = 4;

// Common buffer types
pub type VertexBuffer<T> = Buffer<T, VERTEX>;
pub type IndexBuffer<T> = Buffer<T, INDEX>;
pub type StorageBuffer<T> = Buffer<T, STORAGE>;
pub type UniformBuffer<T> = Buffer<T, UNIFORM>;
pub type IndirectBuffer<T> = Buffer<T, INDIRECT>;

// Internal buffer type since we might use either one
pub enum BufferKind<T: BufferContents> {
    DeviceLocal(DeviceLocalBuffer<T>),
    CpuAccessible(CpuAccessibleBuffer<T>),
}

// An abstraction layer over a valid OpenGL buffer
// This takes a valid OpenGL type and an element type, though the user won't be able make the buffer directly
// This also takes a constant that represents it's OpenGL target
pub struct Buffer<T: BufferContents, const TYPE: u32> {
    buffer: BufferKind<T>,
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

impl<T: BufferContents, const TYPE: u32> Buffer<T, TYPE> {
    // Create a buffer using a slice of elements
    // (will return none if we try to create a zero length Static, Dynamic, or Partial buffer)
    pub fn from_slice(
        graphics: &Graphics,
        slice: &[T],
        mode: BufferMode,
        usage: BufferUsage,
        cmd: &mut AutoCommandBufferBuilder<PrimaryAutoCommandBuffer>,
        //recorder: &Recorder,
    ) -> Option<Self> {
        // Cannot create a zero sized slice for non-resizable buffers or a zero sized buffer in general
        let invalid = slice.is_empty() && !matches!(mode, BufferMode::Resizable);
        let zero = size_of::<T>() == 0;
        if invalid || zero {
            return None;
        }

        // Decompose graphics
        let device = graphics.logical();
        let queue = graphics.queue();

        // Get location and staging buffer location
        log::debug!("Creating buffer with {} elements, stride: {}", slice.len(), size_of::<T>());
        let layout = super::find_optimal_layout(mode, usage);
        log::debug!("Given buffer layout {:?}, optimal layout {:?}", usage, layout);

        
        todo!()
        /*

        // Decompose graphics
        let device = graphics.device();
        let _queues = graphics.queues();

        // Calculate the byte size of the buffer
        let size =
            u64::try_from(size_of::<T>() * slice.len()).unwrap();



        // Create the actual buffer
        let (buffer, allocation) = unsafe {
            device.create_buffer(
                size,
                layout.src_buffer_usage_flags,
                layout.src_buffer_memory_location,
                graphics.queues(),
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
        */
    }

    /*
    // Create an empty buffer if we can (resizable)
    pub fn empty(
        graphics: &Graphics,
        mode: BufferMode,
        usage: BufferUsage,
        recorder: &Recorder,
    ) -> Option<Self> {
        Self::from_slice(graphics, &[], mode, usage, recorder)
    }
    */

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

    /*

    // Convert a range bounds type into the range indices
    // This will return None if the returning indices have a length of 0
    fn convert_range_bounds(
        &self,
        range: impl RangeBounds<usize>,
    ) -> Option<BufferBounds> {
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
            return None;
        }

        if (end - start) == 0 {
            return None;
        }

        Some(BufferBounds {
            offset: start,
            size: end - start,
        })
    }

    // Fills a range in the buffer with a constant value
    pub fn splat_range(
        &mut self,
        _val: T,
        range: impl RangeBounds<usize>,
        _recorder: &Recorder,
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
        recorder: &Recorder,
    ) -> Result<(), BufferError> {
        let collected = iterator.collect::<Vec<_>>();
        self.extend_from_slice(&collected, recorder)
    }

    // Extend the current buffer using data from a new slice
    pub fn extend_from_slice(
        &mut self,
        slice: &[T],
        _recorder: &Recorder,
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
        if !self.usage.permission_host_write {
            return Err(BufferError::InvalidUsage(
                InvalidUsageError::IllegalHostWrite,
            ));
        }

        // Check if we can read from the buffer
        if !self.usage.permission_host_read {
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
        _recorder: &Recorder,
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
        if !self.usage.permission_host_write {
            return Err(BufferError::InvalidUsage(
                InvalidUsageError::IllegalHostWrite,
            ));
        }

        // Get the mapped pointer and write to it the given slice
        todo!()
        Ok(())
    }

    // Read a region of the buffer into a mutable slice immediately
    pub fn read_range(
        &self,
        dst: &mut [T],
        range: impl RangeBounds<usize>,
        _recorder: &Recorder,
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
        if !self.usage.permission_host_read {
            return Err(BufferError::InvalidUsage(
                InvalidUsageError::IllegalHostRead,
            ));
        }

        // Get the mapped pointer and write to it the given slice
        todo!()
        Ok(())
    }

    // Read a region of the buffer into a new vector
    pub fn read_range_as_vec(
        &self,
        range: impl RangeBounds<usize> + Copy,
        recorder: &Recorder,
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
        recorder: &Recorder,
    ) -> Result<Vec<T>, BufferError> {
        self.read_range_as_vec(.., recorder)
    }

    // Clear the buffer contents, resetting the buffer's length down to zero
    pub fn clear(&mut self) {
        // TODO: write to current buffer
        self.length = 0;
        todo!()
    }

    // Copy the data from another buffer's range into this buffer's range
    // dst_offset refers to the offset inside Self
    // src_range refers to the range of 'other'
    pub fn copy_range_from<const OTHER_TYPE: u32>(
        &mut self,
        _src_range: impl RangeBounds<usize>,
        _other: &Buffer<T, OTHER_TYPE>,
        _dst_offset: usize,
        _recorder: &Recorder,
    ) {
        todo!()
    }

    // Copy the data from another buffer into this buffer
    pub fn copy_from<const OTHER: u32>(
        &mut self,
        other: &Buffer<T, OTHER>,
        recorder: &Recorder,
    ) {
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
        recorder: &Recorder,
    ) -> Result<(), BufferError> {
        self.splat_range(val, .., recorder)
    }

    // Overwrite the whole buffer using a slice
    pub fn write(
        &mut self,
        slice: &[T],
        recorder: &Recorder,
    ) -> Result<(), BufferError> {
        self.write_range(slice, .., recorder)
    }

    // Read the whole buffer into a mutable slice
    pub fn read(
        &self,
        slice: &mut [T],
        recorder: &Recorder,
    ) -> Result<(), BufferError> {
        self.read_range(slice, .., recorder)
    }
    */

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