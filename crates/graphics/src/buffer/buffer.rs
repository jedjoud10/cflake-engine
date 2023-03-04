use std::{
    alloc::Layout,
    any::type_name,
    f32::consts::E,
    marker::PhantomData,
    mem::{size_of, ManuallyDrop},
    num::NonZeroU64,
    ops::{Range, RangeBounds},
};

use wgpu::{util::DeviceExt, Maintain, CommandEncoder};

use crate::{
    BufferClearError, BufferCopyError, BufferExtendError,
    BufferInitializationError, BufferMode, BufferNotMappableError,
    BufferReadError, BufferUsage, BufferView, BufferViewMut,
    BufferWriteError, GpuPodRelaxed, Graphics, StagingPool, Vertex,
    R, StagingTarget,
};

// Bitmask from Vulkan BufferUsages
const VERTEX: u32 = wgpu::BufferUsages::VERTEX.bits();
const INDEX: u32 = wgpu::BufferUsages::INDEX.bits();
const STORAGE: u32 = wgpu::BufferUsages::STORAGE.bits();
const UNIFORM: u32 = wgpu::BufferUsages::UNIFORM.bits();
const INDIRECT: u32 = wgpu::BufferUsages::INDIRECT.bits();

// Type of buffer stored as an enum (WGPU BufferUsage)
#[repr(u32)]
#[derive(Copy, Clone, PartialEq, Eq)]
pub enum BufferVariant {
    Vertex = VERTEX,
    Index = INDEX,
    Storage = STORAGE,
    Uniform = UNIFORM,
    Indirect = INDIRECT,
}

// Special vertex buffer (for vertices only)
pub type VertexBuffer<V> = Buffer<<V as Vertex>::Storage, VERTEX>;

// Special triangle (index) buffer (for triangles only)
pub type Triangle<T> = [T; 3];
pub type TriangleBuffer<T> = Buffer<Triangle<T>, INDEX>;

// TODO: Implemenent std430 + std130 for these types of buffers
pub type StorageBuffer<T> = Buffer<T, STORAGE>;
pub type UniformBuffer<T> = Buffer<T, UNIFORM>;

// Indirect command buffer
pub type IndirectBuffer<T> = Buffer<T, INDIRECT>;

// A buffer abstraction over a valid WGPU buffer
// This also takes a constant that represents it's Wgpu target at compile time
// TODO: Handle async read writes and async command buf submissions
pub struct Buffer<T: GpuPodRelaxed, const TYPE: u32> {
    // Raw WGPU buffer
    buffer: wgpu::Buffer,

    // Size / Layout fields
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
    pub fn raw(&self) -> &'a wgpu::Buffer {
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

        // Return an error if the buffer usage flags are invalid
        if usage.contains(BufferUsage::READ) && !usage.contains(BufferUsage::COPY_SRC) {
            return Err(BufferInitializationError::ReadableWithoutCopySrc);
        } else if usage.contains(BufferUsage::WRITE) && !usage.contains(BufferUsage::COPY_DST) {
            return Err(BufferInitializationError::WritableWithoutCopyDst);
        } if mode == BufferMode::Resizable && !usage.contains(BufferUsage::COPY_SRC) {
            return Err(BufferInitializationError::ResizableWithoutCopySrc);
        }

        // Wgpu usages for this buffer
        let wgpu_usages = buffer_usages(variant, usage);

        // Convert the slice into bytes
        let bytes = bytemuck::cast_slice::<T, u8>(slice);

        // Allocate the WGPU buffer
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

        let name = utils::pretty_type_name::<T>();
        log::debug!("Creating buffer [{name}; cap = {capacity}], usage: {wgpu_usages:?}");

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

// Get the buffer usages from the buffer variant and usage wrapper
fn buffer_usages(variant: wgpu::BufferUsages, usage: BufferUsage) -> wgpu::BufferUsages {
    let mut wgpu_usages = variant;
    if usage.contains(BufferUsage::COPY_SRC) {
        wgpu_usages |= wgpu::BufferUsages::COPY_SRC;
    }
    if usage.contains(BufferUsage::COPY_DST) {
        wgpu_usages |= wgpu::BufferUsages::COPY_DST;
    }
    wgpu_usages
}

// Implementation of util methods
impl<T: GpuPodRelaxed, const TYPE: u32> Buffer<T, TYPE> {
    // Get the inner raw WGPU buffer immutably
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
            _ => panic!(),
        }
    }

    // Get the buffer's stride (size of each element)
    pub fn stride(&self) -> usize {
        size_of::<T>()
    }

    // Get the untyped buffer from this typed buffer
    pub fn as_untyped(&self) -> UntypedBuffer {
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
    fn convert_bounds(
        &self,
        range: impl RangeBounds<usize>,
    ) -> Option<(usize, usize)> {
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

// Implementation of safe methods
impl<T: GpuPodRelaxed, const TYPE: u32> Buffer<T, TYPE> {
    // Read from "src" and write to the buffer instantly
    // This is a "fire and forget" command that does not stall the CPU
    // The user can do multiple write calls and expect them to be batched together
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
        if !self.usage.contains(BufferUsage::WRITE) {
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
            &self.graphics,
            StagingTarget::Buffer {
                buffer: &self.buffer,
                offset: (offset * self.stride()) as u64,
                size: (src.len() * self.stride()) as u64,
            },
            bytemuck::cast_slice(src)
        );

        Ok(())
    }

    // Read buffer and write to "dst" instantly
    // Will stall the CPU, since this is waiting for GPU data
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
        if !self.usage.contains(BufferUsage::READ) {
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
            &self.graphics,
            StagingTarget::Buffer {
                buffer: &self.buffer,
                offset: (offset * self.stride()) as u64,
                size: (dst.len() * self.stride()) as u64,
            },

            bytemuck::cast_slice_mut(dst),
        );
        
        Ok(())
    }

    // Clear the buffer and reset it's length
    // This doesn't enqueue a GPU command
    pub fn clear(&mut self) -> Result<(), BufferClearError> {
        if matches!(self.mode, BufferMode::Dynamic) {
            return Err(BufferClearError::IllegalLengthModify);
        }

        self.length = 0;
        Ok(())
    }

    // Fill the buffer with a repeating value specified by "val"
    // This is a "fire and forget" command that does not stall the CPU
    // The user can do multiple splat calls and expect them to be batched together
    pub fn splat(&mut self, val: T) -> Result<(), BufferWriteError> {
        let src = vec![val; self.length];
        self.write(&src, 0)
    }

    // Copy the data from another buffer into this buffer instantly
    // This is a "fire and forget" command that does not stall the CPU
    // The user can do multiple copy_from calls and expect them to be batched together
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
            copy_size
        );
        self.graphics.reuse([encoder]);
        Ok(())
    }

    // Extend this buffer using the given slice instantly
    // This is a "fire and forget" command that does not stall the CPU
    // The user can do multiple extend_from_slice calls and expect them to be batched together
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

        if !self.usage.contains(BufferUsage::WRITE) {
            return Err(BufferExtendError::NonWritable);
        }

        // We know this is valid before hand
        let variant = wgpu::BufferUsages::from_bits(TYPE).unwrap();

        // Wgpu usages for primary buffer
        let usage = buffer_usages(variant, self.usage);

        // Check if we need to allocate a new buffer
        if slice.len() + self.length > self.capacity {
            // Calculate a new capacity and new length
            let capacity = self.capacity + slice.len();
            let capacity = (capacity * 2).next_power_of_two();
            let size = (capacity * self.stride()) as u64;

            // Allocate a new buffer with a higher capacity
            let buffer = self.graphics.device().create_buffer(
                &wgpu::BufferDescriptor {
                    label: None,
                    size,
                    usage,
                    mapped_at_creation: false,
                },
            );

            // Copy the current buffer to the new one
            let mut encoder = self.graphics.acquire();
            encoder.copy_buffer_to_buffer(
                &self.buffer,
                0,
                &buffer,
                0,
                (self.length * self.stride()) as u64,
            );
            self.graphics.reuse([encoder]);
            
            // Swap them out, and drop the last buffer
            let old = std::mem::replace(&mut self.buffer, buffer);
            drop(old);

            // Write using the same encoder
            self.length += slice.len();
            self.write(slice, self.length-slice.len()).unwrap();
        } else {
            // Just write into a sub-part of the buffer
            self.length += slice.len();
            
            // Write using the same encoder
            self.write(slice, self.length-slice.len()).unwrap();
        }

        Ok(())
    }

    // Try to view the buffer immutably immediately
    // Will stall the CPU, since this is synchronous
    pub fn as_view(
        &self,
        bounds: impl RangeBounds<usize>,
    ) -> Result<BufferView<T, TYPE>, BufferNotMappableError> {
        todo!()
        /*
        if !self.usage.contains(BufferUsage::READ) {
            return Err(BufferNotMappableError::AsView);
        }

        // Size and offset of the slice
        let (start, end) = self
            .convert_bounds(bounds)
            .ok_or(BufferNotMappableError::InvalidRange)?;
        let size = (end - start) * self.stride();
        let offset = start * self.stride();

        // Get the staging pool for download
        let staging = self.graphics.staging_pool();
        let data = staging
            .map_read(
                StagingTarget::Buffer(&self.buffer),
                &self.graphics,
                offset as u64,
                size as u64,
            )
            .unwrap();

        Ok(BufferView {
            buffer: &self,
            data,
        })
        */
    }

    // Try to view the buffer mutably (for writing AND reading) immediately
    // Will stall the CPU, since this is synchronous
    // If the BufferUsage is Write only, then reading from BufferViewMut might be slow / might not return buffer contents
    pub fn as_view_mut(
        &mut self,
        bounds: impl RangeBounds<usize>,
    ) -> Result<BufferViewMut<T, TYPE>, BufferNotMappableError> {
        todo!()
        /*
        if !self.usage.contains(BufferUsage::WRITE) {
            return Err(BufferNotMappableError::AsViewMut);
        }

        // Size and offset of the slice
        let (start, end) = self
            .convert_bounds(bounds)
            .ok_or(BufferNotMappableError::InvalidRange)?;
        let size = (end - start) * self.stride();
        let offset = start * self.stride();

        let read = self.usage.contains(BufferUsage::READ);
        let write = self.usage.contains(BufferUsage::WRITE);

        if write && !read {
            // Write only, map staging buffer
            // Get the staging pool for upload
            let staging = self.graphics.staging_pool();
            let data = staging
                .map_write(
                    StagingTarget::Buffer(&self.buffer),
                    &self.graphics,
                    offset as u64,
                    size as u64,
                )
                .unwrap();
            
            Ok(BufferViewMut::Mapped {
                buffer: PhantomData,
                data,
            })
        } else if read && write {
            // Read and write, clone first, then write
            // Create a temporary vector that will store the contents of the buffer
            let mut vector = vec![T::zeroed(); end - start];
            self.read(&mut vector, start).unwrap();

            Ok(BufferViewMut::Cloned {
                buffer: self,
                data: vector,
            })
        } else {
            panic!()
        }
        */
    }

    // Read from "src" and write to the buffer when the encoder is submitted
    // This is a "fire and forget" command that does not stall the CPU
    // The user can do multiple async_write calls and expect them to be batched together
    pub fn async_write(
        &mut self,
        encoder: &mut CommandEncoder,
        src: &[T],
        offset: usize,
    ) -> Result<(), BufferWriteError> {
        todo!()
    }

    // Read buffer and call the callback with the data when done
    // This is not called immediately. Only called when complete
    // The user will not be able to write to the buffer on the GPU or CPU whilst this is in progress
    pub fn async_read(
        &mut self,
        encoder: &mut CommandEncoder,
        bounds: impl RangeBounds<usize>,
        callback: impl FnOnce(&[T]) + Send + Sync,
    ) -> Result<(), BufferReadError> {
        todo!()
    }
}
