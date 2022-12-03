use std::{
    marker::PhantomData,
    mem::{size_of, ManuallyDrop},
    ops::RangeBounds, sync::Arc,
};

use crate::{
    BufferError, Graphics, InvalidModeError,
    InvalidUsageError, Recorder,
};
use bytemuck::{Zeroable, Pod};
use log_err::LogErrResult;
use vulkano::{buffer::{DeviceLocalBuffer, BufferContents, CpuAccessibleBuffer, BufferAccess, BufferAccessObject}, command_buffer::{AutoCommandBufferBuilder, PrimaryCommandBufferAbstract, PrimaryAutoCommandBuffer, CopyBufferInfo}};

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
            hint_device_write: true,
            hint_device_read: true,
            permission_host_write: true,
            permission_host_read: true,
        }
    }
}

pub trait Contents: BufferContents + Pod + Zeroable where [Self]: BufferContents {}
impl<T: BufferContents+ Pod + Zeroable> Contents for T where [T]: BufferContents {}

// Bitmask from WGPU BufferUsages
pub(super) const VERTEX: u32 = 0;
pub(super) const INDEX: u32 = 1;
pub(super) const STORAGE: u32 = 2;
pub(super) const UNIFORM: u32 = 3;
pub(super) const INDIRECT: u32 = 4;

// Common buffer types
pub type VertexBuffer<T> = Buffer<T, VERTEX>;
pub type IndexBuffer<T> = Buffer<T, INDEX>;
pub type StorageBuffer<T> = Buffer<T, STORAGE>;
pub type UniformBuffer<T> = Buffer<T, UNIFORM>;
pub type IndirectBuffer<T> = Buffer<T, INDIRECT>;

// Internal buffer type since we might use either one
pub(super) enum BufferKind<T: BufferContents + Sized> where [T]: BufferContents {
    DeviceLocal(Arc<DeviceLocalBuffer<[T]>>),
    CpuAccessible(Arc<CpuAccessibleBuffer<[T]>>),
}

impl<T: BufferContents + Sized> Clone for BufferKind<T> where [T]: BufferContents {
    fn clone(&self) -> Self {
        match self {
            Self::DeviceLocal(arg0) => Self::DeviceLocal(arg0.clone()),
            Self::CpuAccessible(arg0) => Self::CpuAccessible(arg0.clone()),
        }
    }
}

impl<T: BufferContents + Sized> BufferKind<T> where [T]: BufferContents {
    fn as_buffer_access_object(&self) -> Arc<dyn BufferAccess> {
        match &self {
            BufferKind::DeviceLocal(x) => x.as_buffer_access_object(),
            BufferKind::CpuAccessible(x) => x.as_buffer_access_object(),
        }
    }

    fn as_device_local(&self) -> Arc<DeviceLocalBuffer<[T]>> {
        match self {
            BufferKind::DeviceLocal(x) => x.clone(),
            BufferKind::CpuAccessible(_) => panic!(),
        }
    }

    fn as_cpu_accessible(&self) -> Arc<CpuAccessibleBuffer<[T]>> {
        match self {
            BufferKind::DeviceLocal(_) => panic!(),
            BufferKind::CpuAccessible(x) => x.clone(),
        }
    }
}

// An abstraction layer over a valid OpenGL buffer
// This takes a valid OpenGL type and an element type, though the user won't be able make the buffer directly
// This also takes a constant that represents it's OpenGL target
pub struct Buffer<T: Contents, const TYPE: u32> {
    inner: BufferKind<T>,
    length: usize,
    capacity: usize,
    usage: BufferUsage,
    mode: BufferMode,
    graphics: Graphics,
    _phantom: PhantomData<T>,
}

// Internal bounds used by the buffer
#[derive(Debug, Clone, Copy)]
pub(super) struct BufferBounds {
    offset: usize,
    size: usize,
}

impl<T: Contents, const TYPE: u32> Buffer<T, TYPE> {
    // Create a buffer using a slice of elements
    // (will return none if we try to create a zero length Static, Dynamic, or Partial buffer)
    pub fn from_slice(
        graphics: &Graphics,
        slice: &[T],
        mode: BufferMode,
        usage: BufferUsage,
        recorder: &mut Recorder,
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
        let mut layout = super::find_optimal_layout(mode, usage);
        super::apply_buffer_type(&mut layout, TYPE);
        log::debug!("Given buffer layout {:#?}", usage);
        log::debug!("Optimal layout: {:#?}", layout);
        
        // Create the src buffer
        let inner = super::initialize_buffer::<T>(
            graphics,
            slice.len(),
            layout.src_usage,
            layout.src_host_cached,
            layout.src_kind,
        );

        // If we need to create a temporary staging buffer, do so
        if let Some(usage) = layout.init_usage {
            unsafe {
                // Create a staging buffer if needed
                let staging = CpuAccessibleBuffer::<[T]>::uninitialized_array(
                    graphics.memory_allocator(),
                    slice.len() as u64,
                    usage,
                    false,
                ).expect("Could not create CPU accessible staging buffer");

                // Write to the staging buffer
                let mut write = staging.write().unwrap();
                write.copy_from_slice(slice);

                // Get the main's buffer access
                let dst = inner.as_buffer_access_object();

                // Copy the staging buffer to the main buffer
                let info = CopyBufferInfo::buffers(staging.clone(), dst);
                recorder.internal.copy_buffer(info).unwrap();
            }
        } else {
            // If not, we must write directly to the buffer
            let src = inner.as_cpu_accessible();
            let mut write = src.write().unwrap();
            write.copy_from_slice(slice);
        }

        // Create a buffer instance
        Some(Self {
            inner,
            length: slice.len(),
            capacity: slice.len(),
            mode,
            usage,
            graphics: graphics.clone(),
            _phantom: PhantomData,
        })
        
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

    // Read a region of the buffer into a mutable slice immediately
    pub fn read_range(
        &self,
        dst: &mut [T],
        range: impl RangeBounds<usize>,
    ) -> Result<(), BufferError> {
        let Some(bounds) = self.convert_range_bounds(range) else {
            return Ok(());
        };

        // Check if the given slice matches with the range
        if bounds.size != dst.len() {
            return Err(BufferError::SliceLengthRangeMistmatch(
                dst.len(),
                bounds.size,
            ));
        }

        // Check if we can read from the buffer
        if !self.usage.permission_host_read {
            return Err(BufferError::InvalidUsage(
                InvalidUsageError::IllegalHostRead,
            ));
        }

        match &self.inner {
            BufferKind::DeviceLocal(buffer) => {
                // Use the staging buffer that we created to read back from the buffer
            },
            BufferKind::CpuAccessible(buffer) => {
                // Get a read lock on the buffer
                let read = buffer.read().unwrap();
                let BufferBounds { offset, size } = bounds;
                let src = &read[offset..(offset + size)];
                
                // Read from the CPU accessible buffer
                dst.copy_from_slice(src);
                
                // Logging
                log::debug!("Reading range: {:?} from cpu accessible buffer", bounds);
            },
        }
        Ok(())
    }

    /*
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
    */
    
    // Overwrite a region of the buffer using a slice and a range
    pub fn write_range(
        &mut self,
        src: &[T],
        range: impl RangeBounds<usize>,
    ) -> Result<(), BufferError> {
        let Some(bounds) = self.convert_range_bounds(range) else {
            return Ok(());
        };

        // Check if the given slice matches with the range
        if bounds.size != src.len() {
            return Err(BufferError::SliceLengthRangeMistmatch(
                src.len(),
                bounds.size,
            ));
        }

        // Check if we can write to the buffer
        if !self.usage.permission_host_write {
            return Err(BufferError::InvalidUsage(
                InvalidUsageError::IllegalHostWrite,
            ));
        }

        // Get the mapped pointer and write to it the given slice
        match &self.inner {
            BufferKind::DeviceLocal(_) => todo!(),
            BufferKind::CpuAccessible(buffer) => {
                let mut write = buffer.write().unwrap();
                let BufferBounds { offset, size } = bounds;
                let dst = &mut write[offset..(offset + size)];
                dst.copy_from_slice(src);
                log::debug!("Writing range {:?} to cpu accessible buffer", bounds);
            },
        }
        Ok(())
    }


    /*
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