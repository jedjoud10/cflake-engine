use std::{
    alloc::Layout,
    any::type_name,
    marker::PhantomData,
    mem::{size_of, ManuallyDrop},
};

use wgpu::util::DeviceExt;

use crate::{
    BufferClearError, BufferCopyError, BufferExtendError,
    BufferInitializationError, BufferMode, BufferNotMappableError,
    BufferReadError, BufferUsage, BufferWriteError, GpuPodRelaxed,
    Graphics,
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

        // Panic if the buffer type isn't supported
        let variant = wgpu::BufferUsages::from_bits(TYPE);
        let Some(variant) = variant else {
            return Err(
                BufferInitializationError::InvalidVariantType,
            );
        };

        // TODO: Make this work with the StagingBelt / StagingBuffer

        let wgpu_usages = variant
            | wgpu::BufferUsages::COPY_SRC
            | wgpu::BufferUsages::COPY_DST
            | wgpu::BufferUsages::MAP_READ;

        // Convert the slice into bytes
        let bytes = bytemuck::cast_slice::<T, u8>(slice);

        // Allocate the WGPU buffer
        log::debug!("Allocating raw buffer for type {}, element len: {}, byte len: {}", type_name::<T>(), slice.len(), bytes.len());
        let buffer = graphics.device().create_buffer_init(
            &wgpu::util::BufferInitDescriptor {
                label: None,
                contents: bytes,
                usage: wgpu_usages,
            },
        );
        graphics.device().poll(wgpu::Maintain::Wait);

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
        if !self.usage.contains(BufferUsage::WRITE) {
            return Err(BufferWriteError::InvalidPermissions);
        }

        // Make sure the "offset" doesn't cause writes outside the buffer
        if src.len() + offset > self.length {
            return Err(BufferWriteError::InvalidLen(
                src.len(),
                offset,
                self.len(),
            ));
        }

        // Write to the buffer
        let offset =
            wgpu::BufferAddress::try_from(offset * self.stride())
                .unwrap();
        self.graphics.queue().write_buffer(
            &self.buffer,
            offset,
            bytemuck::cast_slice(src),
        );

        // Wait for the write to happen
        self.graphics.queue().submit([]);
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
        if !self.usage.contains(BufferUsage::READ) {
            return Err(BufferReadError::InvalidPermissions);
        }

        // Make sure the "offset" doesn't cause reads outside the buffer
        if dst.len() + offset > self.length {
            return Err(BufferReadError::InvalidLen(
                dst.len(),
                offset,
                self.len(),
            ));
        }

        let offset = u64::try_from(self.stride() * offset).unwrap();
        let size = u64::try_from(self.stride() * dst.len()).unwrap();

        //TODO: Add this again
        /*
        // Create the staging buffer's descriptor
        let desc = wgpu::BufferDescriptor {
            label: None,
            size,
            usage: wgpu::BufferUsages::COPY_DST | wgpu::BufferUsages::MAP_READ,
            mapped_at_creation: false,
        };

        // Create a temporary staging buffer
        let staging = self.graphics.device().create_buffer(&desc);

        let mut encoder = self.graphics.device().create_command_encoder(&wgpu::CommandEncoderDescriptor {
            label: None,
        });

        // Record the copy command
        encoder.copy_buffer_to_buffer(
            &self.buffer,
            offset,
            &staging,
            0,
            size
        );

        // Submit the encoder to the queue
        self.graphics.queue().submit(Some(encoder.finish()));
        */
        self.graphics.queue().submit([]);

        // Map the buffer (and wait)
        type MapResult = Result<(), wgpu::BufferAsyncError>;
        let (tx, rx) = std::sync::mpsc::channel::<MapResult>();
        let staging_slice = self.buffer.slice(..);

        // Map async (but wait for submission)
        staging_slice.map_async(wgpu::MapMode::Read, move |res| {
            tx.send(res).unwrap()
        });
        self.graphics.device().poll(wgpu::Maintain::Wait);

        // Wait until the buffer is mapped, then read
        if let Ok(Ok(_)) = rx.recv() {
            let bytes = &*staging_slice.get_mapped_range();
            dst.copy_from_slice(bytemuck::cast_slice(bytes));
        }

        Ok(())
    }

    // Clear the buffer and reset it's length
    pub fn clear(&mut self) -> Result<(), BufferClearError> {
        if matches!(self.mode, BufferMode::Dynamic) {
            return Err(BufferClearError::IllegalLengthModify);
        }

        self.length = 0;
        let mut encoder =
            self.graphics.device().create_command_encoder(
                &wgpu::CommandEncoderDescriptor { label: None },
            );
        encoder.clear_buffer(&self.buffer, 0, None);
        self.graphics.queue().submit(Some(encoder.finish()));

        /*
        unsafe {
            self.clear_unchecked();
        }
        */

        Ok(())
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

        /*
        unsafe {
            self.copy_from_unchecked(
                src, dst_offset, src_offset, length,
            );
        }
        */

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

        /*
        unsafe {
            self.extend_from_slice_unchecked(slice);
        }
        */

        Ok(())
    }

    // Try to view the buffer immutably immediately
    pub fn as_slice(&self) -> Result<&[T], BufferNotMappableError> {
        todo!()
    }

    // Try to view the buffer mutably immediately
    pub fn as_slice_mut(
        &mut self,
    ) -> Result<&mut [T], BufferNotMappableError> {
        todo!()
    }
}
