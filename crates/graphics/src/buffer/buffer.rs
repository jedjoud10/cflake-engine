use std::{marker::PhantomData, ops::RangeBounds, mem::{size_of, MaybeUninit}, iter::repeat};
use bytemuck::{Pod, Zeroable};
use wgpu::{util::DeviceExt, BufferUsages};

use crate::{Graphics, Content, BufferViewMut, BufferView};


// Bitmask from WGPU BufferUsages
const VERTEX: u32 = BufferUsages::VERTEX.bits();
const INDEX: u32 = BufferUsages::INDEX.bits();
const STORAGE: u32 = BufferUsages::STORAGE.bits();
const UNIFORM: u32 = BufferUsages::UNIFORM.bits();
const INDIRECT: u32 = BufferUsages::INDIRECT.bits();

// Common buffer types
pub type VertexBuffer<T> = Buffer<T, VERTEX>;
pub type IndexBuffer<T> = Buffer<T, INDEX>;
pub type StorageBuffer<T> = Buffer<T, STORAGE>;
pub type UniformBuffer<T> = Buffer<T, UNIFORM>;
pub type IndirectBuffer<T> = Buffer<T, INDIRECT>;

// An abstraction layer over a valid OpenGL buffer
// This takes a valid OpenGL type and an element type, though the user won't be able make the buffer directly
// This also takes a constant that represents it's OpenGL target
pub struct Buffer<T: Content, const TYPE: u32> {
    buffer: wgpu::Buffer,
    length: usize,
    capacity: usize,
    mode: BufferMode,
    graphics: Graphics,
    _phantom: PhantomData<T>,
}

// Some settings that tell us how exactly we should create the buffer
#[derive(Default, Clone, Copy, PartialEq, Eq, Debug)]
pub enum BufferMode {
    // Dynamic buffers are only created once, but they allow the user to mutate each element
    #[default]
    Dynamic,

    // Partial buffer have a fixed capacity, but a dynamic length
    Parital,

    // Resizable buffers can be resized to whatever length needed
    Resizable,
}

// Internal bounds used by the buffer
pub(super) struct BufferBounds {
    offset: usize,
    size: usize,
}

// Multiple ways of writing to a buffer
// 1. Create a staging buffer, writing to it, then copy
// 2. Create a staging buffer, mapping it and writing to it, then copy
// 3. Write to buffer directly

// Multiple ways of reading from a buffer
// 1. Create a staging buffer as copy dst, copy, then map

impl<T: Content, const TYPE: u32> Buffer<T, TYPE> {
    // Create a buffer using a slice of elements 
    // (will return none if we try to create a zero length Static, Dynamic, or Partial buffer)
    pub fn from_slice(graphics: &Graphics, slice: &[T], mode: BufferMode) -> Option<Self> {
        // Return none if we try to make a null buffer
        if slice.is_empty() && !matches!(mode, BufferMode::Resizable) {
            return None;
        } 

        // Cast slice to appropriate raw data
        let contents = bytemuck::cast_slice::<T, u8>(slice);

        // Create the buffer usage flags
        let usage = {
            // Type bit of the buffer (vertex | uniform | indirect | storage)
            let _type = BufferUsages::from_bits(TYPE).unwrap();

            // Default flags for any type of buffer
            let _default = BufferUsages::COPY_SRC | BufferUsages::COPY_DST;

            _type | _default
        };

        // Create buffer description
        let description = wgpu::util::BufferInitDescriptor {
            label: None,
            contents,
            usage,
        };

        // Create the raw buffer and initialize it
        let buffer = graphics.device().create_buffer_init(&description);
        
        Some(Self {
            buffer,
            length: slice.len(),
            capacity: slice.len(),
            mode,
            graphics: graphics.clone(),
            _phantom: PhantomData,
        })
    }
    

    // Create an empty buffer if we can (resizable)
    pub fn empty(graphics: &Graphics, mode: BufferMode) -> Option<Self> {
        Self::from_slice(graphics, &[], mode)
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
    fn convert_range_bounds(&self, range: impl RangeBounds<usize>) -> Option<BufferBounds> {
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
    pub fn splat_range(&mut self, val: T, range: impl RangeBounds<usize>) {
        if let Some(BufferBounds {
            offset, size 
        }) = self.convert_range_bounds(range) {
            // Create the contents based on the value and size
            let contents = vec![val; size];
            let data = bytemuck::cast_slice(&contents);
            
            // Write into the buffer the specified contents
            let offset = offset as u64 * self.stride() as u64;
            self.graphics.queue().write_buffer(
                &self.buffer,
                offset,
                data
            );
        }
    }

    // Extent the current buffer using data from an iterator
    pub fn extend_from_iterator<I: Iterator<Item = T>>(&mut self, iterator: I) {
        todo!()
    }

    // Extend the current buffer using data from a new slice
    pub fn extend_from_slice(&mut self, slice: &[T]) {
        todo!()
    }

    // Overwrite a region of the buffer using a slice and a range
    pub fn write_range(&mut self, slice: &[T], range: impl RangeBounds<usize>) {
        let Some(BufferBounds {
            offset, size 
        }) = self.convert_range_bounds(range) else {
            return;
        };
        assert_eq!(
            size,
            slice.len(),
            "Buffer write range is not equal to slice length"
        );

        // Convert to bytes
        let offset = u64::try_from(self.stride() * offset).unwrap(); 

        // Write to the buffer
        self.graphics.queue().write_buffer(
            &self.buffer,
            offset,
            bytemuck::cast_slice(slice),
        );
    }

    // Read a region of the buffer into a mutable slice immediately
    pub fn read_range(&self, slice: &mut [T], range: impl RangeBounds<usize>) {
        let Some(BufferBounds {
            offset, size 
        }) = self.convert_range_bounds(range) else {
            return;
        };

        // Convert to bytes
        let offset = u64::try_from(self.stride() * offset).unwrap(); 
        let size = u64::try_from(self.stride() * size).unwrap();

        // Create the staging buffer's descriptor
        let desc = wgpu::BufferDescriptor {
            label: None,
            size: self.buffer.size(),
            usage: BufferUsages::COPY_DST | BufferUsages::MAP_READ,
            mapped_at_creation: false,
        };

        // Create a temporary staging buffer
        let staging = self.graphics.device().create_buffer(&desc);


        // Create a command encoder
        let mut encoder = self.graphics.device().create_command_encoder(
            &Default::default()
        );

        // Record the copy command
        encoder.copy_buffer_to_buffer(
            &self.buffer,
            offset,
            &staging,
            0,
            size);

        // Submit the encoder to the queue
        self.graphics.queue().submit([encoder.finish()]);
        self.graphics.device().poll(wgpu::Maintain::Wait);

        // Map the buffer asnychronously
        type MapResult = Result<(), wgpu::BufferAsyncError>;
        let (tx, rx) = 
            std::sync::mpsc::sync_channel::<MapResult>(1);
        let staging_slice = staging.slice(..);
        staging_slice.map_async(wgpu::MapMode::Read, move |res| tx.send(res).unwrap());
            
        // Wait for the buffer mapping
        if let Ok(value) = rx.recv() {
            value.unwrap();

            // Copy to the output now
            let staging = &*staging_slice.get_mapped_range();
            slice.copy_from_slice(bytemuck::cast_slice(staging));
        }
    }

    // Read a region of the buffer into a new vector
    pub fn read_range_as_vec(&self, range: impl RangeBounds<usize> + Copy) -> Vec<T> {
        let Some(BufferBounds {
            size, .. 
        }) = self.convert_range_bounds(range) else {
            return Vec::new();
        };

        // Create a vec and read into it
        let mut vec = vec![T::zeroed(); size];
        self.read_range(
            &mut vec,
            range,
        );
        vec
    }

    // Read the whole buffer into a new vector
    pub fn read_to_vec(&self) -> Vec<T> {
        self.read_range_as_vec(..)
    }

    // Clear the buffer contents, resetting the buffer's length down to zero
    pub fn clear(&mut self) {
        let mut encoder = self.graphics.device().create_command_encoder(&Default::default());
        encoder.clear_buffer(&self.buffer, 0, None);
        self.graphics.queue().submit([encoder.finish()]);
        self.length = 0;        
    }

    // Copy the data from another buffer's range into this buffer's range
    // dst_offset refers to the offset inside Self
    // src_range refers to the range of 'other'
    pub fn copy_range_from<const OTHER_TYPE: u32>(
        &mut self,
        src_range: impl RangeBounds<usize>,
        other: &Buffer<T, OTHER_TYPE>,
        dst_offset: usize,
    ) {
        todo!()
    }

    // Copy the data from another buffer into this buffer
    pub fn copy_from<const OTHER: u32>(&mut self, other: &Buffer<T, OTHER>) {
        assert_eq!(
            self.len(),
            other.len(),
            "Cannot copy from buffer, length mismatch"
        );

        self.copy_range_from(.., other, 0);
    }

    // Fills the whole buffer with a constant value
    pub fn splat(&mut self, val: T) {
        self.splat_range(val, ..)
    }

    // Overwrite the whole buffer using a slice
    pub fn write(&mut self, slice: &[T]) {
        self.write_range(slice, ..)
    }

    // Read the whole buffer into a mutable slice
    pub fn read(&self, slice: &mut [T]) {
        self.read_range(slice, ..)
    }

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
}