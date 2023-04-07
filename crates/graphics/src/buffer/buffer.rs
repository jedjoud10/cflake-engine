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

use crate::{
    BufferClearError, BufferCopyError, BufferExtendError, BufferInfo,
    BufferInitializationError, BufferMode, BufferNotMappableError,
    BufferReadError, BufferSplatError, BufferUsage, BufferView,
    BufferViewMut, BufferWriteError, DispatchIndirect,
    DrawIndexedIndirect, DrawIndirect, GpuPod, Graphics, StagingPool,
    Vertex, R,
};

// Bitmask from Vulkan BufferUsages
const VERTEX: u32 = wgpu::BufferUsages::VERTEX.bits();
const INDEX: u32 = wgpu::BufferUsages::INDEX.bits();
const UNIFORM: u32 = wgpu::BufferUsages::UNIFORM.bits();
const INDIRECT: u32 = wgpu::BufferUsages::INDIRECT.bits();

// Type of buffer stored as an enum (WGPU BufferUsage)
#[repr(u32)]
#[derive(Copy, Clone, PartialEq, Eq)]
pub enum BufferVariant {
    Vertex = VERTEX,
    Index = INDEX,
    Uniform = UNIFORM,
    Indirect = INDIRECT,
    None,
}

// Special vertex buffer (for vertices only)
pub type VertexBuffer<V> = Buffer<<V as Vertex>::Storage, VERTEX>;

// Special triangle (index) buffer (for triangles only)
pub type Triangle<T> = [T; 3];
pub type TriangleBuffer<T> = Buffer<Triangle<T>, INDEX>;

// TODO: Implemenent std430 + std130 for these types of buffers
pub type UniformBuffer<T> = Buffer<T, UNIFORM>;

// Indirect buffers for GPU rendering
pub type DrawIndirectBuffer = Buffer<DrawIndirect, INDIRECT>;
pub type DrawIndexedIndirectBuffer =
    Buffer<DrawIndexedIndirect, INDIRECT>;
pub type DispatchIndirectBuffer = Buffer<DispatchIndirect, INDIRECT>;

// A buffer abstraction over a valid WGPU buffer
// This also takes a constant that represents it's Wgpu target at compile time
// TODO: Handle async read writes and async command buf submissions
pub struct Buffer<T: GpuPod, const TYPE: u32 = 0> {
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

// PartialEq implementation
impl<T: GpuPod, const TYPE: u32> PartialEq for Buffer<T, TYPE> {
    fn eq(&self, other: &Self) -> bool {
        self.buffer.global_id() == other.buffer.global_id() &&
        self.length == other.length &&
        self.capacity == other.capacity &&
        self.usage == other.usage &&
        self.mode == other.mode
    }
}

// Buffer initialization
impl<T: GpuPod, const TYPE: u32> Buffer<T, TYPE> {
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

        // Return an error if the buffer usage flags are invalid
        if usage.contains(BufferUsage::READ)
            && !usage.contains(BufferUsage::COPY_SRC)
        {
            return Err(
                BufferInitializationError::ReadableWithoutCopySrc,
            );
        } else if usage.contains(BufferUsage::WRITE)
            && !usage.contains(BufferUsage::COPY_DST)
        {
            return Err(
                BufferInitializationError::WritableWithoutCopyDst,
            );
        }
        if mode == BufferMode::Resizable
            && !usage.contains(BufferUsage::COPY_SRC)
        {
            return Err(
                BufferInitializationError::ResizableWithoutCopySrc,
            );
        }

        // Wgpu usages for this buffer
        let wgpu_usages = buffer_usages(variant, usage)?;

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

    // Create a buffer that contains zeroed out data
    pub fn zeroed(
        graphics: &Graphics,
        length: usize,
        mode: BufferMode,
        usage: BufferUsage,
    ) -> Result<Self, BufferInitializationError> {
        let vec = vec![T::zeroed(); length];
        Self::from_slice(graphics, &vec, mode, usage)
    }

    // Creates a buffer that contains one element repeated multiple times
    pub fn splatted(
        graphics: &Graphics,
        length: usize,
        value: T,
        mode: BufferMode,
        usage: BufferUsage,
    ) -> Result<Self, BufferInitializationError> {
        let vec = vec![value; length];
        Self::from_slice(graphics, &vec, mode, usage)
    }
}

// Get the buffer usages from the buffer variant and usage wrapper
fn buffer_usages(
    variant: Option<wgpu::BufferUsages>,
    usage: BufferUsage,
) -> Result<wgpu::BufferUsages, BufferInitializationError> {
    // If the user does not specify a usage and there isn't a valid variant, return error
    let mut base = if let Some(variant) = variant {
        variant
    } else {
        if usage.is_empty() {
            return Err(
                BufferInitializationError::UnkownBufferUsageOrType,
            );
        } else {
            wgpu::BufferUsages::empty()
        }
    };

    // map the wrapper BufferUsages to wgpu BufferUsages
    if usage.contains(BufferUsage::COPY_SRC) {
        base |= wgpu::BufferUsages::COPY_SRC;
    }
    if usage.contains(BufferUsage::COPY_DST) {
        base |= wgpu::BufferUsages::COPY_DST;
    }
    if usage.contains(BufferUsage::STORAGE) {
        base |= wgpu::BufferUsages::STORAGE;
    }

    Ok(base)
}

// Implementation of util methods
impl<T: GpuPod, const TYPE: u32> Buffer<T, TYPE> {
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
            UNIFORM => BufferVariant::Uniform,
            INDIRECT => BufferVariant::Indirect,
            _ => BufferVariant::None,
        }
    }

    // Get the buffer's stride (size of each element)
    pub fn stride(&self) -> usize {
        size_of::<T>()
    }

    // Get the untyped buffer from this typed buffer
    pub fn as_untyped(&self) -> BufferInfo {
        BufferInfo {
            buffer: &self.buffer,
            length: self.len(),
            stride: self.stride(),
            capacity: self.capacity(),
            usage: self.usage(),
            mode: self.mode(),
            variant: self.variant(),
        }
    }

    // Validate the bounds of the RangeBounds trait into (start, end)
    pub fn convert_bounds_to_indices(
        &self,
        range: impl RangeBounds<usize>,
    ) -> Option<(usize, usize)> {
        let start = match range.start_bound() {
            Bound::Included(start) => *start,
            Bound::Excluded(_) => panic!(),
            Bound::Unbounded => 0,
        };

        let end = match range.end_bound() {
            Bound::Included(end) => *end + 1,
            Bound::Excluded(end) => *end,
            Bound::Unbounded => self.length,
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

    // Convert the bounds of the RangeBounds trait into a wgpu BufferBinding
    pub(crate) fn convert_bounds_to_binding(
        &self,
        range: impl RangeBounds<usize>,
    ) -> Option<wgpu::BufferBinding> {
        // Full range, exit early
        if range.start_bound() == Bound::Unbounded
            && range.end_bound() == range.start_bound()
        {
            return Some(self.buffer.as_entire_buffer_binding());
        }

        // Custom range, gotta make sure it's valid
        let (start, end) = self.convert_bounds_to_indices(range)?;

        // Calculate byte offset and size (if needed)
        let offset = (start * self.stride()) as u64;
        let size = (end != self.len()).then(|| {
            let size = (start * self.stride()) as u64;
            NonZeroU64::new(size).unwrap()
        });

        Some(wgpu::BufferBinding {
            buffer: &self.buffer,
            offset,
            size,
        })
    }
}

// Implementation of safe methods
impl<T: GpuPod, const TYPE: u32> Buffer<T, TYPE> {
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
    pub fn async_read<'a>(
        &'a self,
        range: impl RangeBounds<usize>,
        callback: impl FnOnce(&[T]) + Sync + Send,
    ) {
        todo!()
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

    // Fill a buffer region with a repeating value specified by "val"
    // This is a "fire and forget" command that does not stall the CPU
    // The user can do multiple splat calls and expect them to be batched together
    pub fn splat(
        &mut self,
        range: impl RangeBounds<usize>,
        val: T,
    ) -> Result<(), BufferSplatError> {
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
            copy_size,
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
        let usage = buffer_usages(Some(variant), self.usage).unwrap();

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
            self.write(slice, self.length - slice.len()).unwrap();
        } else {
            // Just write into a sub-part of the buffer
            self.length += slice.len();

            // Write using the same encoder
            self.write(slice, self.length - slice.len()).unwrap();
        }

        Ok(())
    }

    // Try to view the buffer immutably immediately
    // Will stall the CPU, since this is synchronous
    pub fn as_view(
        &self,
        bounds: impl RangeBounds<usize>,
    ) -> Result<BufferView<T, TYPE>, BufferNotMappableError> {
        if !self.usage.contains(BufferUsage::READ) {
            return Err(BufferNotMappableError::AsView);
        }

        // Size and offset of the slice
        let (start, end) =
            self.convert_bounds_to_indices(bounds).ok_or(
                BufferNotMappableError::InvalidRange(self.length),
            )?;
        let size = (end - start) * self.stride();
        let offset = start * self.stride();

        // Get the staging pool for download
        let staging = self.graphics.staging_pool();
        let data = staging
            .map_buffer_read(
                &self.graphics,
                &self.buffer,
                offset as u64,
                size as u64,
            )
            .unwrap();

        Ok(BufferView {
            buffer: &self,
            data,
        })
    }

    // Try to view the buffer mutably (for writing AND reading) immediately
    // Will stall the CPU, since this is synchronous
    // If the BufferUsage is Write only, then reading from BufferViewMut might be slow / might not return buffer contents
    pub fn as_view_mut(
        &mut self,
        bounds: impl RangeBounds<usize>,
    ) -> Result<BufferViewMut<T, TYPE>, BufferNotMappableError> {
        if !self.usage.contains(BufferUsage::WRITE) {
            return Err(BufferNotMappableError::AsViewMut);
        }

        // Size and offset of the slice
        let (start, end) =
            self.convert_bounds_to_indices(bounds).ok_or(
                BufferNotMappableError::InvalidRange(self.length),
            )?;
        let size = (end - start) * self.stride();
        let offset = start * self.stride();

        // Check if we can read the buffer
        let read = self.usage.contains(BufferUsage::READ);

        if !read {
            // Write only buffer view, uses QueueWriteBufferView
            let staging = self.graphics.staging_pool();
            let data = staging
                .map_buffer_write(
                    &self.graphics,
                    &self.buffer,
                    offset as u64,
                    size as u64,
                )
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
