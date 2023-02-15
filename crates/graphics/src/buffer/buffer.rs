use std::{
    alloc::Layout,
    any::type_name,
    marker::PhantomData,
    mem::{size_of, ManuallyDrop}, f32::consts::E, num::NonZeroU64, ops::{Range, RangeBounds},
};

use wgpu::{util::DeviceExt, Maintain};

use crate::{
    BufferClearError, BufferCopyError, BufferExtendError,
    BufferInitializationError, BufferMode, BufferNotMappableError,
    BufferReadError, BufferUsage, BufferWriteError, GpuPodRelaxed,
    Graphics, R, BufferView, BufferViewMut, StagingPool,
};

// Bitmask from Vulkan BufferUsages
const VERTEX: u32 = wgpu::BufferUsages::VERTEX.bits();
const INDEX: u32 = wgpu::BufferUsages::INDEX.bits();
const STORAGE: u32 = wgpu::BufferUsages::STORAGE.bits();
const UNIFORM: u32 = wgpu::BufferUsages::UNIFORM.bits();
const INDIRECT: u32 = wgpu::BufferUsages::INDIRECT.bits();

// Type of buffer stored as an enum (Vulkan BufferUsages)
#[repr(u32)]
#[derive(Copy, Clone, PartialEq, Eq)]
pub enum BufferVariant {
    Vertex = VERTEX,
    Index = INDEX,
    Storage = STORAGE,
    Uniform = UNIFORM,
    Indirect = INDIRECT,
}

// Common buffer types
pub type VertexBuffer<T> = Buffer<T, VERTEX>;
pub type Triangle<T> = [T; 3];
pub type TriangleBuffer<T> = Buffer<Triangle<T>, INDEX>;
pub type StorageBuffer<T> = Buffer<T, STORAGE>;
pub type UniformBuffer<T> = Buffer<T, UNIFORM>;
pub type IndirectBuffer<T> = Buffer<T, INDIRECT>;

// A buffer abstraction over a valid WGPU buffer
// This also takes a constant that represents it's Wgpu target at compile time
// TODO: Handle async read writes and async command buf submissions
pub struct Buffer<T: GpuPodRelaxed, const TYPE: u32> {
    // Raw WGPU buffer
    buffer: wgpu::Buffer,

    // Size fields
    length: usize,
    capacity: usize,

    // Legal Permissions
    usage: BufferUsage,
    mode: BufferMode,
    _phantom: PhantomData<T>,

    // Keep the graphics API alive
    graphics: Graphics,
}

// Untyped buffer that does not contain a generic type nor type ID
pub struct UntypedBuffer<'a> {
    buffer: &'a wgpu::Buffer,
    variant: BufferVariant,
    length: usize,
    stride: usize,
    capacity: usize,
    usage: BufferUsage,
    mode: BufferMode,
}

impl<'a> UntypedBuffer<'a> {
    // Get the inner raw WGPU buffer
    pub fn raw(&self) -> &wgpu::Buffer {
        self.buffer
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

    // Get the buffer usage
    pub fn usage(&self) -> BufferUsage {
        self.usage
    }

    // Get the buffer mode
    pub fn mode(&self) -> BufferMode {
        self.mode
    }

    // Get the buffer's stride (length of each element)
    pub fn stride(&self) -> usize {
        self.stride
    }

    // Get the buffer variant type
    pub fn variant(&self) -> BufferVariant {
        self.variant
    }
}

// Buffer initialization
impl<T: GpuPodRelaxed, const TYPE: u32> Buffer<T, TYPE> {
    // Try to create a buffer with the specified mode, usage, and slice data
    pub fn from_slice(
        graphics: &Graphics,
        slice: &[T],
        mode: BufferMode,
        usage: BufferUsage,
    ) -> Result<Self, BufferInitializationError> {
        // Cannot create a zero sized stride buffer
        assert!(
            size_of::<T>() > 0,
            "Buffers do not support zero-sized types"
        );

        // Cannot create a zero sized slice if we aren't resizable
        if slice.is_empty() && !matches!(mode, BufferMode::Resizable)
        {
            return Err(
                BufferInitializationError::EmptySliceNotResizable,
            );
        }

        // Return an error if the buffer type isn't supported
        let variant = wgpu::BufferUsages::from_bits(TYPE);
        let Some(variant) = variant else {
            return Err(
                BufferInitializationError::InvalidVariantType,
            );
        };

        // Wgpu usages for primary buffer 
        let wgpu_usages = variant
            | wgpu::BufferUsages::COPY_SRC
            | wgpu::BufferUsages::COPY_DST;

        // Convert the slice into bytes
        let bytes = bytemuck::cast_slice::<T, u8>(slice);

        // Allocate the primary WGPU buffer
        let buffer = graphics.device().create_buffer_init(
            &wgpu::util::BufferInitDescriptor {
                label: None,
                contents: bytes,
                usage: wgpu_usages,
            },
        );
        
        // Calculate the number of elements that can fit in this one allocation
        let stride = size_of::<T>() as u64;
        let capacity = (buffer.size() / stride) as usize;

        // Create the struct and return it
        Ok(Self {
            length: slice.len(),
            capacity,
            mode,
            usage,
            _phantom: PhantomData,
            buffer,
            graphics: graphics.clone(),
        })
    }

    // Create a buffer with a specific capacity and a length of 0
    pub fn with_capacity<'a>(
        graphics: &Graphics,
        capacity: usize,
        mode: BufferMode,
        usage: BufferUsage,
    ) -> Result<Self, BufferInitializationError> {
        let vec = vec![T::zeroed(); capacity];
        let mut buffer =
            Self::from_slice(graphics, &vec, mode, usage)?;
        buffer.length = 0;
        Ok(buffer)
    }
}

// Implementation of util methods
impl<T: GpuPodRelaxed, const TYPE: u32> Buffer<T, TYPE> {
    // Get the inner raw WGPU buffer
    pub fn raw(&self) -> &wgpu::Buffer {
        &self.buffer
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

    // Get the buffer usage
    pub fn usage(&self) -> BufferUsage {
        self.usage
    }

    // Get the buffer mode
    pub fn mode(&self) -> BufferMode {
        self.mode
    }

    // Get the buffer variant (TYPE wrapper)
    pub fn variant(&self) -> BufferVariant {
        match TYPE {
            VERTEX => BufferVariant::Vertex,
            INDEX => BufferVariant::Index,
            STORAGE => BufferVariant::Storage,
            UNIFORM => BufferVariant::Uniform,
            INDIRECT => BufferVariant::Indirect,
            _ => panic!("This shouldn't happen. Fuck me"),
        }
    }

    // Get the buffer's stride (length of each element)
    pub fn stride(&self) -> usize {
        size_of::<T>()
    }

    // Get the untyped buffer from this typed buffer
    pub fn untyped(&self) -> UntypedBuffer {
        UntypedBuffer {
            buffer: &self.buffer,
            length: self.len(),
            stride: self.stride(),
            capacity: self.capacity(),
            usage: self.usage(),
            mode: self.mode(),
            variant: self.variant(),
        }
    }

    // Convert the bounds of the RangeBounds trait
    fn convert_bounds(&self, range: impl RangeBounds<usize>) -> Option<(usize, usize)> {
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

        Some((start, end))
    } 
}

// Implementation of unsafe methods
impl<T: GpuPodRelaxed, const TYPE: u32> Buffer<T, TYPE> {
    // Transmute the buffer into another type of buffer unsafely
    pub unsafe fn transmute<U: GpuPodRelaxed>(
        self,
    ) -> Buffer<U, TYPE> {
        assert_eq!(
            Layout::new::<T>(),
            Layout::new::<U>(),
            "Layout type mismatch, cannot transmute buffer"
        );

        Buffer::<U, TYPE> {
            buffer: self.buffer,
            length: self.length,
            capacity: self.capacity,
            usage: self.usage,
            mode: self.mode,
            _phantom: PhantomData,
            graphics: self.graphics.clone(),
        }
    }
}

// Implementation of safe methods
impl<T: GpuPodRelaxed, const TYPE: u32> Buffer<T, TYPE> {
    // Read from "src" and write to buffer instantly
    pub fn write(
        &mut self,
        src: &[T],
        offset: usize,
    ) -> Result<(), BufferWriteError> {
        // Nothing to read from
        if src.is_empty() {
            return Ok(());
        }

        // Make sure we can write to the buffer
        if self.usage != BufferUsage::Write && self.usage != BufferUsage::ReadWrite {
            return Err(BufferWriteError::NonWritable);
        }

        // Make sure the "offset" doesn't cause writes outside the buffer
        if src.len() + offset > self.length {
            return Err(BufferWriteError::InvalidLen(
                src.len(),
                offset,
                self.len(),
            ));
        }

        // Use the staging pool for data writes
        let staging = self.graphics.staging_pool();
        staging.write(
            &self.buffer,
            &self.graphics,
            (offset * self.stride()) as u64,
            bytemuck::cast_slice(src)
        );

        Ok(())
    }

    // Read buffer and write to "dst" instantly
    pub fn read<'a>(
        &'a self,
        dst: &mut [T],
        offset: usize,
    ) -> Result<(), BufferReadError> {
        // Nothing to write to
        if dst.is_empty() {
            return Ok(());
        }

        // Make sure we can read from the buffer
        if self.usage != BufferUsage::Read && self.usage != BufferUsage::ReadWrite {
            return Err(BufferReadError::NonReadable);
        }

        // Make sure the "offset" doesn't cause reads outside the buffer
        if dst.len() + offset > self.length {
            return Err(BufferReadError::InvalidLen(
                dst.len(),
                offset,
                self.len(),
            ));
        }

        // Use the staging pool for data reads 
        let staging = self.graphics.staging_pool();
        staging.read(
            &self.buffer,
            &self.graphics,
            (offset * self.stride()) as u64,
            bytemuck::cast_slice_mut(dst)
        );

        Ok(())
    }

    // Clear the buffer and reset it's length
    pub fn clear(&mut self) -> Result<(), BufferClearError> {
        if matches!(self.mode, BufferMode::Dynamic) {
            return Err(BufferClearError::IllegalLengthModify);
        }

        self.length = 0;
        let mut encoder = self.graphics.acquire();
        encoder.clear_buffer(&self.buffer, 0, None);
        self.graphics.submit([encoder]);
        Ok(())
    }


    // Fill the buffer with a repeating value specified by "val"
    pub fn splat(
        &mut self,
        val: T
    ) -> Result<(), BufferWriteError> {
        let src = vec![val; self.length];
        self.write(&src, 0)
    }

    // Copy the data from another buffer into this buffer instantly
    pub fn copy_from<const TYPE2: u32>(
        &mut self,
        src: &Buffer<T, TYPE2>,
        dst_offset: usize,
        src_offset: usize,
        length: usize,
    ) -> Result<(), BufferCopyError> {
        if length == 0 {
            return Ok(());
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

        todo!();
        Ok(())
    }

    // Extend this buffer using the given slice instantly
    pub fn extend_from_slice(
        &mut self,
        slice: &[T],
    ) -> Result<(), BufferExtendError> {
        if slice.is_empty() {
            return Ok(());
        }

        if matches!(self.mode, BufferMode::Dynamic) {
            return Err(BufferExtendError::IllegalLengthModify);
        }

        if slice.len() + self.length > self.capacity
            && matches!(self.mode, BufferMode::Parital)
        {
            return Err(BufferExtendError::IllegalReallocation);
        }

        if self.usage != BufferUsage::Write && self.usage != BufferUsage::ReadWrite {
            return Err(BufferExtendError::NonWritable);
        }

        // We know this is valid before hand
        let variant = wgpu::BufferUsages::from_bits(TYPE).unwrap();

        // Wgpu usages for primary buffer 
        let usage = variant
            | wgpu::BufferUsages::COPY_SRC
            | wgpu::BufferUsages::COPY_DST;

        // Check if we need to allocate a new buffer
        if slice.len() + self.length > self.capacity {            
            // Calculate a new capacity and new length
            let capacity = self.capacity + slice.len();
            let size = (capacity * self.stride()) as u64;

            // Allocate a new buffer with a higher capacity
            let buffer = self.graphics.device().create_buffer(&wgpu::BufferDescriptor {
                label: None,
                size,
                usage,
                mapped_at_creation: false,
            });

            // Copy the current buffer to the new one
            let mut encoder = self.graphics.acquire();
            encoder.copy_buffer_to_buffer(
                &self.buffer,
                0,
                &buffer,
                0,
                (self.length * self.stride()) as u64
            );

            // Wait till the copy finishes
            self.graphics.submit([encoder]);
            self.graphics.device().poll(Maintain::Wait);

            // Swap them out, and drop the last buffer
            let old = std::mem::replace(&mut self.buffer, buffer);
            drop(old)
        } else {
            // Just read into a sub-part of the buffer
            self.write(
                slice,
                self.length,
            ).unwrap();
        }

        Ok(())
    }

    // Try to view the buffer immutably immediately
    pub fn as_view(&self, bounds: impl RangeBounds<usize>) -> Result<BufferView<T, TYPE>, BufferNotMappableError> {
        if self.usage != BufferUsage::Read && self.usage != BufferUsage::ReadWrite {
            return Err(BufferNotMappableError::AsView);
        }

        // Size and offset of the slice
        let (start, end) = self.convert_bounds(bounds)
            .ok_or(BufferNotMappableError::InvalidRange)?;
        let size = (end - start) * self.stride();
        let offset = start * self.stride();

        // Get the staging pool for download
        let staging = self.graphics.staging_pool();
        let data = staging.map_read(
            &self.buffer,
            &self.graphics,
            offset as u64,
            size as u64
        ).unwrap();

        Ok(BufferView {
            buffer: &self,
            data,
        })
    }

    // Try to view the buffer mutably (for writing AND reading) immediately
    // If the BufferUsage is Write only, then reading from BufferViewMut might be slow / might not return buffer contents
    pub fn as_view_mut(&mut self, bounds: impl RangeBounds<usize>) -> Result<BufferViewMut<T, TYPE>, BufferNotMappableError> {
        if self.usage != BufferUsage::ReadWrite && self.usage != BufferUsage::Write {
            return Err(BufferNotMappableError::AsViewMut);
        }

        // Size and offset of the slice
        let (start, end) = self.convert_bounds(bounds)
            .ok_or(BufferNotMappableError::InvalidRange)?;
        let size = (end - start) * self.stride();
        let offset = start * self.stride();

        match self.usage {
            // Write only, map staging buffer
            BufferUsage::Write => {
                // Get the staging pool for upload
                let staging = self.graphics.staging_pool();
                let data = staging.map_write(
                    &self.buffer,
                    &self.graphics,
                    offset as u64,
                    size as u64
                ).unwrap();
            
                Ok(BufferViewMut::Mapped {
                    buffer: PhantomData,
                    data
                })
            },

            // Read and write, clone first, then write
            BufferUsage::ReadWrite => {
                // Create a temporary vector that will store the contents of the buffer
                let mut vector = vec![T::zeroed(); end - start];
                self.read(&mut vector, start).unwrap();
            
                Ok(BufferViewMut::Cloned {
                    buffer: self,
                    data: vector,
                })
            },

            _ => panic!()
        }
    }
}
